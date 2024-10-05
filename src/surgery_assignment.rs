use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::collections::BTreeMap;
use rand;
use minilp::Variable;
use minilp::{Problem, OptimizationDirection, ComparisonOp};
use crate::builder::Instance;


/*
function assign using LP relaxation (instance, surgeon?) -> 
    vector of Vecdeques for this surgeon {
    
    //prepare for solver:
    // ##### actually can initiate this in lp_solver. let dict for variable values, with keys (patient_idx, day) //where day valid for patient
    let dict{patiend_idx: dict{day: weight}} for weights, with same keys
    lp_relaxation_solver(capacities, durations, weight dict)
    }
*/

// returns vec with instance.days+1 entries (final entry is unassigned patients)
pub fn lp_relax_day_assign_per_surgeon(instance: &Instance, surgeon_idx: usize)
    -> Result<Vec<VecDeque<usize>>, String> {
        let capacities = &instance.surgeons[surgeon_idx].max_surgery_time;
        let durations: Vec<u16> = instance.patients.iter().map(|x| x.surgery_duration).collect();

        // Setting up LP
        let mut variable_dict: BTreeMap<usize, BTreeMap<usize, Variable>> = BTreeMap::new(); //X_{patient_idx, day}
        let mut problem = Problem::new(OptimizationDirection::Minimize);
        for patient_idx in 0..instance.patients.len() {
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
                    if day < instance.days {day - first_day} else {30} as f64, (0.0, 1.0)));
            }
            
            // Introduce patient spread constraints
            problem.add_constraint((first_day..=final_day).map(|day| (*(patient_dict.get_mut(&day).unwrap()), 1.0)), 
                ComparisonOp::Eq, 1.0);
            }

        //Introduce capacity constraints
        for day in 0..instance.days {
            let variable_duration_pair_iter = (0..instance.patients.len()).filter(|patient_idx| variable_dict.get(patient_idx).unwrap().contains_key(&day))
            .map(|patient_idx| (*(variable_dict.get(&patient_idx).unwrap().get(&day).unwrap()), durations[patient_idx] as f64));
            problem.add_constraint(variable_duration_pair_iter, ComparisonOp::Le, capacities[day] as f64);
        }

        // Solve LP
        let solution_of_lp = problem.solve().unwrap();

        //Order patients according to entropy of distribution of their corresponding variables. This is prep for next step       
        let mut patient_entropy_vector: Vec<(usize, f64)> = Vec::new();
        for patient_idx in 0..instance.patients.len() {
            let patient_dict = variable_dict.get(&patient_idx).unwrap();

            let mut running_entropy = 0.0;
            for idx_day_variable in patient_dict {
                running_entropy -= solution_of_lp[*(idx_day_variable.1)] * solution_of_lp[*(idx_day_variable.1)].log2()
            }

            patient_entropy_vector.push((patient_idx, running_entropy));
        }
        patient_entropy_vector.sort_by(|a, b| if a.1 <= b.1 {Ordering::Less} else {Ordering::Greater});

        //##### Perhaps modify this to take the bast of a few attempts, or to implement some backtracking
        //Assign patients randomly according to day dist.
        let mut patient_assignment_vec: Vec<VecDeque<usize>> = vec![VecDeque::new(); instance.days];
        let mut available_capacities = capacities.clone();
        for (patient_idx, _) in &patient_entropy_vector {
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
                            assert!(conditional_divider > 0.0, "conditional divider became <= 0")
                        }
                    }
                }
                // If reached here, means patient is not optional and all days are cancelled for them
                return Err("LP rounding did not work".into());
            }
        }

        Ok(patient_assignment_vec)
    }