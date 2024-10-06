use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::path::is_separator;
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
        let capacities = &instance.surgeons[surgeon_idx].max_surgery_time;
        let durations: Vec<u16> = instance.patients.iter().map(|x| x.surgery_duration).collect();
        let is_mandatory: Vec<bool> = (0..durations.len()).map(|patient_idx| instance.patients[patient_idx].mandatory).collect();

        #[cfg(test)]
        println!("number of days: {:?}", capacities.len());
        #[cfg(test)]
        println!("capacities: {capacities:?}");
        #[cfg(test)]
        println!("number of patients: {:?}, of which mandatory: {:?}", durations.len(), is_mandatory.iter().map(|x| u8::from(*x)).sum::<u8>());
        #[cfg(test)]
        println!("durations: {:?}", durations);

        // Setting up LP
        let mut variable_dict: BTreeMap<usize, BTreeMap<usize, Variable>> = BTreeMap::new(); //X_{patient_idx, day}
        let mut problem = Problem::new(OptimizationDirection::Minimize);
        for patient_idx in 0..durations.len() {
            let patient_ref = &instance.patients[patient_idx];
            let first_day = patient_ref.surgery_release_day;
            let final_day = if !patient_ref.mandatory {
                instance.days
            } else {
                patient_ref.surgery_due_day
            };
            variable_dict.insert(patient_idx, BTreeMap::new());
            let patient_dict = variable_dict.get_mut(&patient_idx).unwrap();
            
            // Introduce variables
            for day in first_day..=final_day {
                patient_dict.insert(day, problem.add_var(
                    (if day < instance.days {day - first_day} else {80}) as f64, (0.0, 1.0)));
            }
            
            // Introduce patient spread constraints
            problem.add_constraint((first_day..=final_day).map(|day| (*(patient_dict.get(&day).unwrap()), 1.0)), 
                ComparisonOp::Eq, 1.0);
            }

        //Introduce capacity constraints
        for day in 0..instance.days {
            let variable_duration_pair_iter = (0..durations.len()).filter(|patient_idx| variable_dict.get(patient_idx).unwrap().contains_key(&day))
            .map(|patient_idx| (*(variable_dict.get(&patient_idx).unwrap().get(&day).unwrap()), durations[patient_idx] as f64));
            problem.add_constraint(variable_duration_pair_iter, ComparisonOp::Le, capacities[day] as f64);
        }

        // Solve LP
        let solver_result = problem.solve();
        let Ok(solution_of_lp) = solver_result else {
            return Err(format!("Solver didn't work. Gave error: {}", solver_result.err().unwrap()));
        };
        #[cfg(test)]
        println!("{solution_of_lp:?}");

        //Order patients according to entropy of distribution of their corresponding variables. This is prep for next step       
        let mut patient_entropy_vector: Vec<(usize, f64)> = Vec::new();
        for patient_idx in 0..instance.patients.len() {
            let patient_dict = variable_dict.get(&patient_idx).unwrap();

            let mut running_entropy = 0.0;
            for idx_day_variable in patient_dict {
                let variable_value =  solution_of_lp[*(idx_day_variable.1)];
                if !(variable_value > 0.0) {continue;}
                running_entropy -= variable_value * variable_value.log2()
            }

            patient_entropy_vector.push((patient_idx, running_entropy));
        }
        patient_entropy_vector.sort_by(|a, b| {
            //first order my mandatory
            if is_mandatory[a.0] != is_mandatory[b.0] {
                is_mandatory[b.0].cmp(&is_mandatory[a.0])

            //second by entropy
            } else {
                if a.1 <= b.1 {Ordering::Less} else {Ordering::Greater}
            }
        });

        //##### Perhaps modify this to take the bast of a few attempts, or to implement some backtracking
        //Assign patients randomly according to day dist.
        let mut patient_assignment_vec: Vec<VecDeque<usize>> = vec![VecDeque::new(); instance.days + 1];
        let mut available_capacities = capacities.clone();
        for (patient_idx, _) in &patient_entropy_vector {
            #[cfg(test)]
            if *patient_idx < 20 {
                println!("Patient {}, variables: {:?}, first day: {}", patient_idx,
                variable_dict.get(&patient_idx).unwrap().iter().map(|x| solution_of_lp[*(x.1)]).collect::<Vec<f64>>(),
                instance.patients[*patient_idx].surgery_release_day);
            }

            let duration = durations[*patient_idx];
            let mut conditional_divider = 1.0;
            let mut cancelled_days: HashMap<usize, ()> = HashMap::new();
            'outer: loop {
                let random_num: f64 = rand::random();
                let mut cumul_prob = 0.0;
                'inner: for (day, var) in variable_dict.get(patient_idx).unwrap() {
                    if cancelled_days.contains_key(day) {
                        continue 'inner;
                    }
                    cumul_prob += solution_of_lp[*var] / conditional_divider;
                    if random_num < cumul_prob {
                        if *day == instance.days {
                            patient_assignment_vec[*day].push_back(*patient_idx);
                            break 'outer;
                        } else if duration <= available_capacities[*day] {
                            patient_assignment_vec[*day].push_back(*patient_idx);
                            available_capacities[*day] -= duration;
                            break 'outer;
                        } else {
                            cancelled_days.insert(*day, ());
                            conditional_divider -= solution_of_lp[*var];
                            //If all non-zero days are cancelled, assign to last day:
                            if !(conditional_divider > 0.0) {
                                if !is_mandatory[*patient_idx] {
                                    patient_assignment_vec[instance.days].push_back(*patient_idx);
                                    break 'outer;
                                } else {
                                    return Err(format!("Was unable to assign mandatory patient {}", patient_idx));
                                }
                            }
                            continue 'outer;
                        }
                    }
                }
                // If reached here, means patient is not optional and all days are cancelled for them
                return Err("LP rounding did not work".into());
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
        builder::instance_build(r#"C:\Users\chenv\ihtc2024chen\public_datasets\i01.json"#);

        let Ok(instance) = result else {
            panic!("{}", result.err().unwrap());
        };
        
        let result = lp_relaxation_surgery_knapsack(&instance, 0);
        let Ok(patients_per_day_per_surgeon) = result else{
            panic!("{}", result.err().unwrap());
        };
    }
}