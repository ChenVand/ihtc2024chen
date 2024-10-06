use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::path::is_separator;
use itertools::Itertools;
use rand;
use minilp::{Variable, Problem, OptimizationDirection, ComparisonOp};
use crate::builder::{Instance, Patient};

enum SurgeryKnapsackSolver {
    LPRelaxation,
    DynamicByDay
}

// returns vec with instance.days+1 entries (final entry is unassigned patients)
pub fn lp_relaxation_surgery_knapsack(instance: &Instance, surgeon_idx: usize)
    -> Result<Vec<VecDeque<usize>>, String> {
        let bump_penalty = 80;
        let patients = &instance.patients;
        let capacities = &instance.surgeons[surgeon_idx].max_surgery_time;
        let relevant_patient_idxs = (0..patients.len()).filter(|idx| 
            patients[*idx].surgeon_id == instance.surgeons[surgeon_idx].id).collect::<Vec<usize>>();

        //#####TRASH
        // let durations: Vec<u16> = patients.iter().map(|x| x.surgery_duration).collect();
        // let is_mandatory: Vec<bool> = relevant_patient_idxs.iter().map(|patient_idx| patients[*patient_idx].mandatory).collect();

        #[cfg(test)]
        println!("number of days: {:?}", capacities.len());
        #[cfg(test)]
        println!("capacities: {capacities:?}");
        #[cfg(test)]
        println!("number of patients: {:?}, of which mandatory: {:?}", relevant_patient_idxs.len(), 
            relevant_patient_idxs.iter().filter(|&&idx| instance.patients[idx].mandatory).collect_vec().len());
        // #[cfg(test)]
        // println!("durations: {:?}", relevant_patient_idxs.iter().map(|&idx| patients[idx].surgery_duration).collect_vec());

        // Setting up LP
        let mut variable_dict: BTreeMap<usize, BTreeMap<usize, Variable>> = BTreeMap::new(); //X_{patient_idx, day}
        let mut problem = Problem::new(OptimizationDirection::Minimize);
        for patient_idx in relevant_patient_idxs.iter() {
            let patient_ref = &patients[*patient_idx];
            let first_day = patient_ref.surgery_release_day;
            let final_day = if !patient_ref.mandatory {
                instance.days
            } else {
                patient_ref.surgery_due_day
            };
            variable_dict.insert(*patient_idx, BTreeMap::new());
            let patient_dict = variable_dict.get_mut(patient_idx).unwrap();
            
            // Introduce variables
            for day in first_day..=final_day {
                patient_dict.insert(day, problem.add_var(
                    (if day < instance.days {day - first_day} else {bump_penalty}) as f64, (0.0, 1.0)));
            }
            
            // Introduce patient spread constraints
            problem.add_constraint((first_day..=final_day).map(|day| (*(patient_dict.get(&day).unwrap()), 1.0)), 
                ComparisonOp::Eq, 1.0);
            }

        //Introduce capacity constraints
        for day in 0..instance.days {
            let variable_duration_pair_iter = relevant_patient_idxs.iter().filter(|&&idx| variable_dict.get(&idx).unwrap().contains_key(&day))
            .map(|&idx| (*(variable_dict.get(&idx).unwrap().get(&day).unwrap()), patients[idx].surgery_duration as f64));

            problem.add_constraint(variable_duration_pair_iter, ComparisonOp::Le, capacities[day] as f64);
        }

        // Solve LP
        let solver_result = problem.solve();
        let Ok(solution_of_lp) = solver_result else {
            return Err(format!("Solver didn't work. Gave error: {}", solver_result.err().unwrap()));
        };
        #[cfg(test)]
        println!("{solution_of_lp:?}");

        //Sort patients according to entropy of distribution of their corresponding variables. This order will be used in the rounding of next step       
        let mut patient_entropy_vector: Vec<(usize, f64)> = Vec::new();
        for patient_idx in relevant_patient_idxs.iter() {
            let patient_dict = variable_dict.get(patient_idx).unwrap();

            let mut running_entropy = 0.0;
            'inner: for idx_day_variable in patient_dict {
                let variable_value =  solution_of_lp[*(idx_day_variable.1)];
                if !(variable_value > 0.0) {continue 'inner;}
                running_entropy -= variable_value * variable_value.log2()
            }

            patient_entropy_vector.push((*patient_idx, running_entropy));
        }

        // #[cfg(test)]
        // println!("length of entropy vector before sorting: {}", patient_entropy_vector.len());
        patient_entropy_vector.sort_by(|a, b| {
            //first order my mandatory
            if patients[a.0].mandatory != patients[b.0].mandatory {
                patients[b.0].mandatory.cmp(&patients[a.0].mandatory)

            //second by entropy
            } else {
                if a.1 <= b.1 {Ordering::Less} else {Ordering::Greater}
            }
        });

        //##### Perhaps modify this to take the bast of a few attempts, or to implement some backtracking
        //Assign patients randomly according to day dist.
        let mut patient_assignment_vec: Vec<VecDeque<usize>> = vec![VecDeque::new(); instance.days + 1];
        let mut available_capacities = capacities.clone();
        for &(patient_idx, _) in &patient_entropy_vector {
            #[cfg(test)]
            if true {
                println!("Patient {}, variables: {:?}, first day: {}", patient_idx,
                variable_dict.get(&patient_idx).unwrap().iter().map(|x| solution_of_lp[*(x.1)]).collect::<Vec<f64>>(),
                instance.patients[patient_idx].surgery_release_day);
            }

            let duration = patients[patient_idx].surgery_duration;
            let mut conditional_divider = 1.0;
            let mut cancelled_days: HashMap<usize, ()> = HashMap::new();
            //Attempt rounding. Attempts will fail if not enough capacity in chosen day
            'outer: loop {
                let random_num: f64 = rand::random();
                let mut cumul_prob = 0.0;
                'inner: for (day, var) in variable_dict.get(&patient_idx).unwrap() {
                    if cancelled_days.contains_key(day) {
                        continue 'inner;
                    }
                    cumul_prob += solution_of_lp[*var] / conditional_divider;
                    if random_num < (cumul_prob - 1e-10) {
                        if *day == instance.days {
                            patient_assignment_vec[*day].push_back(patient_idx);
                            break 'outer;
                        } else if duration <= available_capacities[*day] {
                            patient_assignment_vec[*day].push_back(patient_idx);
                            available_capacities[*day] -= duration;
                            break 'outer;
                        } else {
                            cancelled_days.insert(*day, ());
                            conditional_divider -= solution_of_lp[*var];
                            //If all non-zero days are cancelled, assign to last day:
                            if !(conditional_divider > 0.0) {
                                if !patients[patient_idx].mandatory {
                                    patient_assignment_vec[instance.days].push_back(patient_idx);
                                    break 'outer;
                                } else {
                                    //All positive-probability days for mandatory patient were cancelled 
                                    return Err(format!("Was unable to assign mandatory patient {}", patient_idx));
                                }
                            }
                            continue 'outer;
                        }
                    }
                }
                //########## Why do I reach here?
                #[cfg(test)]
                println!("Random number was {random_num}");
                return Err(format!("Unable to assign for for patient {}, with details: {:?}, and remaining capacities{:?}.", patient_idx, patients[patient_idx],
                    available_capacities));
            }
        }
        #[cfg(test)]
        println!("{patient_assignment_vec:?}");

        Ok(patient_assignment_vec)
    }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder;
    use itertools::Itertools;

    #[test]
    fn check_lp_relax_day_assign_per_surgeon() {
        let result =
        builder::instance_build(r#"C:\Users\chenv\ihtc2024chen\public_datasets\i12.json"#);

        let Ok(instance) = result else {
            panic!("{}", result.err().unwrap());
        };
        
        let result = lp_relaxation_surgery_knapsack(&instance, 0);
        let Ok(patients_per_day_per_surgeon) = result else{
            panic!("{}", result.err().unwrap());
        };
    }
}