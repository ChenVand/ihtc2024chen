use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::channel;
use std::path::is_separator;
use itertools::Itertools;
use rand;
use minilp::{Variable, Problem, OptimizationDirection, ComparisonOp};
use crate::builder::{Instance, Patient};

enum SurgeryKnapsackSolver {
    LPRelaxation,
    DynamicByDay
}

//##### This is also the initializer function for the struct
//##### incorporate lock_info, to accommodate patient bumping
//##### Consider giving higher weights to earlier days, due to successive locking
// returns vec with instance.days+1 entries (final entry is unassigned patients)
pub fn lp_relaxation_surgery_knapsack(instance: &Instance, surgeon_idx: usize, )
    -> Result<Vec<VecDeque<usize>>, String> {
        let patients = &instance.patients;
        let capacities = &instance.surgeons[surgeon_idx].max_surgery_time;
        let relevant_patient_idxs = (0..patients.len()).filter(|idx| 
            patients[*idx].surgeon_id == instance.surgeons[surgeon_idx].id).collect::<Vec<usize>>();

        // Define weights for minimization function
        let mandatory_multiplier = 5;
        let bump_weight = 50;
        let weight_func = |idx: usize, day: usize| {
            if patients[idx].mandatory {
                (mandatory_multiplier * (day - patients[idx].surgery_release_day)) as f64
            } else {
                (if day < instance.days {day - patients[idx].surgery_release_day} else {bump_weight}) as f64
            }
        };

        /*
        #[cfg(test)]
        println!("number of days: {:?}", capacities.len());
        #[cfg(test)]
        println!("capacities: {capacities:?}");
        #[cfg(test)]
        println!("number of patients: {:?}, of which mandatory: {:?}", relevant_patient_idxs.len(), 
            relevant_patient_idxs.iter().filter(|&&idx| instance.patients[idx].mandatory).collect_vec().len());
        */
        

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
                patient_dict.insert(day, problem.add_var(weight_func(*patient_idx, day), (0.0, 1.0)));
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
            #[cfg(test)]
            println!("LP solver didn't work for surgeon {}, returned error {}.", surgeon_idx, solver_result.clone().err().unwrap());

            return Err(format!("Solver didn't work. Gave error: {}", solver_result.err().unwrap()));
        };

        /*
        #[cfg(test)]
        println!("{solution_of_lp:?}");
        */

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

        patient_entropy_vector.sort_by(|a, b| {
            //first order my mandatory
            if patients[a.0].mandatory != patients[b.0].mandatory {
                patients[b.0].mandatory.cmp(&patients[a.0].mandatory)

            //second by entropy
            } else {
                if a.1 <= b.1 {Ordering::Less} else {Ordering::Greater}
            }
        });

        //Assign patients randomly according to distribution of day variables of this patient.
        let mut patient_assignment_vec: Vec<VecDeque<usize>> = vec![VecDeque::new(); instance.days + 1];
        let mut available_capacities = capacities.clone();
        for &(patient_idx, _) in &patient_entropy_vector {
            /*
            #[cfg(test)]
            if true {
                println!("Patient {}, variables: {:?}, first day: {}", patient_idx,
                variable_dict.get(&patient_idx).unwrap().iter().map(|x| solution_of_lp[*(x.1)]).collect::<Vec<f64>>(),
                instance.patients[patient_idx].surgery_release_day);
            }
            */
            
            let duration = patients[patient_idx].surgery_duration;
            let mut conditional_divider = 1.0;
            let mut cancelled_days: HashMap<usize, ()> = HashMap::new();
            //Attempt rounding. Attempts will fail if not enough capacity in chosen day
            'outer: loop {
                //##### look into seed
                let random_num: f64 = rand::random();
                let mut cumul_prob = 0.0;
                'inner: for (day, var) in variable_dict.get(&patient_idx).unwrap() {
                    if cancelled_days.contains_key(day) {
                        continue 'inner;
                    }
                    cumul_prob += solution_of_lp[*var] / conditional_divider;
                    if random_num < (cumul_prob + 1e-6) {
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
                            if !(conditional_divider > 1e-6) {
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
                
                /*
                #[cfg(test)]
                println!("Rounding failed for surgeon {surgeon_idx}");
                #[cfg(test)]
                println!("Random number was {random_num}");
                */

                return Err(format!("Unable to assign for for patient {}, with details: {:?}. Remaining capacities{:?}.", patient_idx, patients[patient_idx],
                    available_capacities));
            }
        }
        
        //Squeeze in bumped patients if possible
        loop {
            if patient_assignment_vec[instance.days].len() == 0 {break;}

            //patient_assignment_vec[instance.days] is a VecDeque of indices (in instance.patients) of patients that were not assigned.
            let (j, min_duration) = patient_assignment_vec[instance.days].iter().enumerate().map(|(j, &idx)| (j, patients[idx].surgery_duration)).min_by(
                |a, b| a.1.cmp(&b.1)
            ).unwrap();
            //available_capacities has length instance.days
            let (day, &max_capacity) = available_capacities.iter().enumerate().max_by(|a, b| a.1.cmp(&b.1)).unwrap();
            
            if min_duration <= max_capacity {
                //patient patient_assignment_vec[instance.days][j] can be assigned to day.
                let moved_patient_idx = patient_assignment_vec[instance.days].remove(j).unwrap();
                /*
                #[cfg(test)]
                println!("squeezing patient {moved_patient_idx} to day {day}");
                */

                patient_assignment_vec[day].push_back(moved_patient_idx);
                available_capacities[day] -= min_duration;
            } else {
                //No patient can be squeezed.
                break;
            }
        }

        #[cfg(test)]
        println!("Unassigned patients for surgeon {surgeon_idx}: {:?}", patient_assignment_vec[instance.days].iter().map(|&idx| (idx, patients[idx].surgery_duration)).collect_vec());
        #[cfg(test)]
        println!("remaining capacities for surgeon {surgeon_idx}: {available_capacities:?}");

        Ok(patient_assignment_vec)
    }

//#####Return assignments.
//##### return array where first idx is by day, not surgeon.
//#####Important: return also LP problem: Problem, so that additional constraints can be added due to OT/room infeasibility
pub fn assign_surgery_days(instance: &Instance) {
    let num_threads = 4;
    let surgeon_idx: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let (tx, rx) = channel();

    //will collect assignments with surgeon index
    let mut assignments: Vec<(usize, Vec<VecDeque<usize>>)> = Vec::new();
    //######## Also collect errors
    //let mut errors

    thread::scope(|s| {
        for _ in 0..num_threads {
            let (idx, tx) = (Arc::clone(&surgeon_idx), tx.clone());
            let instance_1 = instance;
            s.spawn(move || {
                loop {
                    let mut current_surgeon_idx = idx.lock().unwrap();
                    if *current_surgeon_idx == instance.surgeons.len() {
                        break;
                    }
                    let surgeon_idx = current_surgeon_idx.clone();
                    *current_surgeon_idx += 1;
                    
                    tx.send((surgeon_idx, lp_relaxation_surgery_knapsack(instance_1, surgeon_idx))).unwrap();
                }
            });
        }

        // Collect results here
        for _ in 0..instance.surgeons.len() {
            let (surgeon_idx, lp_knapsack_result) = rx.recv().unwrap();
            if lp_knapsack_result.is_ok() {
                assignments.push((surgeon_idx, lp_knapsack_result.ok().unwrap()));
            }
            //#######handle errors.
        }

        #[cfg(test)]
        println!("{} surgeons succeeded out of {}", assignments.len(), instance.surgeons.len());
    });
}

//#####disallow patient from being assigned to current or passed days. Also, lock patients in previous days.
pub fn bump_patient() {}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder;
    use itertools::Itertools;

    fn prepper() -> Instance {
        let result =
        builder::instance_build(r#"C:\Users\chenv\ihtc2024chen\public_datasets\i13.json"#);

        let Ok(instance) = result else {
            panic!("{}", result.err().unwrap());
        };

        instance
    }

    #[test]
    fn check_assign_surgery_days() {
        let instance = prepper();

        assign_surgery_days(&instance);
    }

    #[test]
    fn check_lp_relax_day_assign_per_surgeon() {
        let instance = prepper();
        let surgeon_idx: usize = 1;
        
        let result = lp_relaxation_surgery_knapsack(&instance, surgeon_idx);
        let Ok(patients_per_day) = result else{
            panic!("{}", result.err().unwrap());
        };

        for (day, day_deque) in patients_per_day.iter().enumerate() {
            if day == instance.days {
                continue;
            }

            let mut patient_duration_sum: u16 = 0;
            for patient_idx in day_deque {
                let patient_ref = &instance.patients[*patient_idx];
                assert!(patient_ref.surgery_release_day <= day, "assigned day is before release day");
                assert!(patient_ref.surgery_due_day >= day, "assigned day is after due day");
                patient_duration_sum += patient_ref.surgery_duration;
            }
            assert!(patient_duration_sum <= instance.surgeons[surgeon_idx].max_surgery_time[day], 
            "surgeon max_surgery_time exceeded");
        }

        for &idx in patients_per_day[instance.days].iter() {
            assert!(!instance.patients[idx].mandatory, "A mandatory patient was bumped.");
        }

        // //At most 10 patients were bumped
        // assert!(patients_per_day[instance.days].len() < 10);
    }
}