use crate::builder::{Instance, Patient};
use crate::builder;
use std::collections::VecDeque;
// use std::iter::Iterator;

// creates assignment of patients per day per surgeon. usize refers to index in 
// instance.patients
fn prelim_day_assignment(instance: &Instance) -> Vec<Vec<VecDeque<usize>>> {
    // output: patient_day_per_surgeon
    let mut patients_per_day_per_surgeon: Vec<Vec<VecDeque<usize>>> = Vec::new();

    let mut patient_indices: Vec<usize> = (0..instance.patients.len()).collect();
    
    for surgeon_idx in 0..instance.surgeons.len() {
        let surgeon_id = &instance.surgeons[surgeon_idx].id;

        // add row corresponding to surgeon
        patients_per_day_per_surgeon.push(vec![VecDeque::new(); instance.days]);

        //going through patient_indices in decreasing order
        for j in (0..patient_indices.len()).rev() {
            let patient_index = patient_indices[j];
            if &instance.patients[patient_index].surgeon_id == surgeon_id {
                patients_per_day_per_surgeon[surgeon_idx][instance.patients[patient_index].surgery_release_day].push_back(patient_index);
                patient_indices.remove(j);
            }
        }
    }
    assert!(patient_indices.len() == 0, "patient_indices wasn't exhausted during initial assignment of days");

    // Order each VecDeque<usize> in increasing *distance* to due date, and then increasing surgery_duration

    for surgeon_idx in 0..instance.surgeons.len() {
        for day in 0..instance.days {
            sort_patients_in_slot(instance, &mut patients_per_day_per_surgeon[surgeon_idx][day])
        }
    }

    patients_per_day_per_surgeon
}

// ##patients ordering is in decreasing surgery duration, and then increasing *distance* to due date
fn sort_patients_in_slot(instance: &Instance, patient_indices: &mut VecDeque<usize>) {
    if patient_indices.len() <= 1 {
        return ();
    }

    let mut rhs: VecDeque<usize> = VecDeque::new();

    let pivot = patient_indices.pop_front().unwrap();
    let pivot_due_date = instance.patients[pivot].surgery_due_day;
    let pivot_duration = instance.patients[pivot].surgery_duration;

    for j in (0..patient_indices.len()).rev() {
        let curr_patient = &instance.patients[patient_indices[j]];

        if curr_patient.surgery_due_day > pivot_due_date || 
        ((curr_patient.surgery_due_day == pivot_due_date) && (curr_patient.surgery_duration < pivot_duration)) {
            rhs.push_back(patient_indices.remove(j).unwrap());
        }
    }

    sort_patients_in_slot(instance, patient_indices);
    patient_indices.push_back(pivot);

    sort_patients_in_slot(instance, &mut rhs);
    patient_indices.append(&mut rhs);
}

//To be parallelized
fn arrange_patients_for_surgeons(instance: &Instance, patients_per_day_per_surgeon: &mut Vec<Vec<VecDeque<usize>>>, surgeon_spec: Option<&[usize]>) -> Result<Vec<VecDeque<usize>>, String> {
    // #[cfg(test)]
    // println!("instance is: {:#?}", instance);
    
    let num_surgeons = instance.surgeons.len();
    let mut unassigned_patients: Vec<VecDeque<usize>> = vec![VecDeque::new(); num_surgeons];
    
    match surgeon_spec {
        Some(surgeon_slice) => {
            for surgeon_idx in surgeon_slice {
                let result = dynamic_by_day_surgery_knapsack(&instance.surgeons[*surgeon_idx].max_surgery_time, &mut patients_per_day_per_surgeon[*surgeon_idx], 
                    0, &instance.patients)?;
                unassigned_patients[*surgeon_idx] = result;
            }
        },
        None => {
            for surgeon_idx in 0..num_surgeons {
                #[cfg(test)]
                println!("I am in a test for surgeon {}", surgeon_idx);

                let result = dynamic_by_day_surgery_knapsack(&instance.surgeons[surgeon_idx].max_surgery_time, &mut patients_per_day_per_surgeon[surgeon_idx], 
                    0, &instance.patients)?;
                unassigned_patients[surgeon_idx] = result;
            }
        },
    }
    Ok(unassigned_patients)
}

fn dynamic_by_day_surgery_knapsack(capacity: &Vec<u16>, assignment: &mut Vec<VecDeque<usize>>, first_day: usize, patients: &Vec<Patient>) 
    -> Result<VecDeque<usize>, String> {

    assert!(first_day < capacity.len());

    'days: for day in first_day..capacity.len() {
        #[cfg(test)]
        println!("Entering day {}", day);
        /*
        for start point:
            while duration exceeds capacity, successively accumulate patients to bump.
            bump these patients and apply function recursively
            if this did not succeed, undo bump and continue to next iteration. 
        */

        // if there are no patients assigned today (so capacity trivially satisfied)
        if assignment[day].len() == 0 {
            continue 'days;
        }

        'attempts: for _attempt in 0..assignment[day].len() {
            let mut summed_duration = Iterator::sum::<u16>(assignment[day].iter().map(|x| patients[*x].surgery_duration)); 

            if summed_duration <= capacity[day] {continue 'days;}

            let mut patients_to_bump: VecDeque<usize> = VecDeque::new(); 
            
            // collect patients to be bumped
            'inner: for patient_counter in (0..assignment[day].len()).rev() {
                if summed_duration > capacity[day] {
                    patients_to_bump.push_front(assignment[day].remove(patient_counter).unwrap());
                    summed_duration -= patients[patients_to_bump[0]].surgery_duration;
                } else {
                    break 'inner;
                }
            }

            // if all patients that could, have been moved and still capacity is not satisfied, then abort.
            if summed_duration > capacity[day] {
                return Err(format!("Capacity in day {:?} cannot be reached", day));
            }

            // bump patients to the next day, or into the abyss if this is the last day
            //##### modify to remember bumped
            if day < capacity.len()-1 {
                for bumped_patient in patients_to_bump {
                    assignment[day + 1].push_front(bumped_patient);
                }
                let result = dynamic_by_day_surgery_knapsack(capacity, assignment, day+1, patients);
                if result.is_ok() {
                    return result;
                } else {
                    // return bumped patients
                    let mut returned_patient: usize;
                    for j in (0..assignment[day+1].len()).rev() {
                        if patients[assignment[day+1][j]].surgery_release_day <= day {
                            returned_patient = assignment[day+1].remove(j).unwrap();
                            assignment[day].push_front(returned_patient);
                        }
                    }
                }
            } else if day == capacity.len()-1 {
                return Ok(patients_to_bump);
            } else {
                return Err("day exceeded final day somehow".into());
            }    
        }
        // If this is reached, then all attempts failed for day
        return Err(format!("all attempts failed for day {}", day));

        /*//Previous attempt
        // loop over different attempts (corresponding to different values of start_p) to successfully bump patients out of day
        let mut start_p: usize = assignment[day].len();
        loop {
            let mut demanded_duration = Iterator::sum::<u16>(assignment[day].iter().map(|x| patients[*x].surgery_duration));          
            // check if any patients even need bumping. Done inside loop because should be refreshed for every start_p
            #[cfg(test)]
            println!("for start_p = {}, Demanded duration is {}, capacity is {}", start_p, demanded_duration, capacity[day]);

            if demanded_duration <=  capacity[day] {break;}
            let mut patients_to_bump: VecDeque<usize> = VecDeque::new();

            // collect patients to be bumped
            'inner: for patient_counter in (0..(start_p-1)).rev() {
                // #[cfg(test)]
                // println!("patient_counter = {}", patient_counter);
                if demanded_duration > capacity[day] {
                    // #[cfg(test)]
                    // println!("demanded_duration = {} is still too high", demanded_duration);
                    if patients[assignment[day][patient_counter]].surgery_due_day > day {
                        // note patients_to_bump is decreasing
                        patients_to_bump.push_back(patient_counter);
                        demanded_duration -= patients[assignment[day][patient_counter]].surgery_duration;
                    }
                } else {
                    break 'inner;
                }                
            }

            // if all patients that could, have been moved and still capacity is not satisfied, then abort.
            if demanded_duration > capacity[day] {
                return Err(String::from(format!("Capacity in day {:?} cannot be reached", day)));
            }


            // bump patients into the abyss if the last day
            //###### perhaps modify to keep a list of patients not assigned a day
            if day == capacity.len()-1 {
                let mut unassigned_patients = VecDeque::new();

                for patient_local_idx in patients_to_bump.clone() {
                    unassigned_patients.push_back(assignment[day].remove(patient_local_idx).unwrap());
                }
                return Ok(unassigned_patients);
            }

            // bump patients (bumped_patient is actually patient index in instance.patients)
            // patient_local_idx decreases
            for patient_local_idx in patients_to_bump.clone() {
                let bumped_patient = assignment[day].remove(patient_local_idx).unwrap();
                assignment[day + 1].push_front(bumped_patient);
            }

            // try recursion, and if it fails, undo bumping to allow for next attempt with lower start_p
            let result = dynamic_by_day_surgery_knapsack(capacity, assignment, day + 1, patients);
            if result.is_ok() {
                // note that this means that the outermost day loop stops, and so it only goes up to a problematic day.
                return result;
            } else {
                // patient_local_idx increases
                for patient_local_idx in patients_to_bump.into_iter().rev() {
                    let returned_patient = assignment[day+1].pop_front().unwrap();

                    // this insertion is dangerous if there is an error. however another method (like pushing to the front)
                    // would require a different method to start_p. Perhaps this is actually cleaner, although if there are 
                    // many unbumpable patients (which seems not to be the case) then in this alternative method we could be 
                    // performing he same computation again and again.
                    assignment[day].insert(patient_local_idx, returned_patient);
                }
            }

            if start_p > 1 {start_p -= 1;} else {return Err("start_p reached end and no valid assignment was found".into());}
        }  */

    }
    //If this was reached, then all days were valid and no-one needed bumping
    return Ok(VecDeque::new()); 
        
}

//##### todo
fn dynamic_by_patient_surgery_knapsack(capacity: &Vec<u16>, assignment: &mut Vec<VecDeque<usize>>, first_day: usize, patients: &Vec<Patient>) 
    -> Result<VecDeque<usize>, String> {todo!()}

//##### todo
fn lp_relaxation_surgery_knapsack(capacity: &Vec<u16>, assignment: &mut Vec<VecDeque<usize>>, first_day: usize, patients: &Vec<Patient>) 
    -> Result<VecDeque<usize>, String> {todo!()}

//##### todo
fn bump_patient(patient_idx: usize) -> Result<(),()> {todo!();}

//##### implement patient bumping of doesn't work
fn patient_OT_assignment_for_day(instance: &Instance, patients_per_day_per_surgeon: &mut Vec<Vec<VecDeque<usize>>>, day: usize) -> Result<Vec<Vec<usize>>, String> {
    // outer Vec corresponds to OTs as ordered in instance.operating_theaters

    let mut items: Vec<(usize, u16)> = vec![];
    for surgeon_vec in & *patients_per_day_per_surgeon {
        for patient_idx in &surgeon_vec[day] {
            items.push((*patient_idx, instance.patients[*patient_idx].surgery_duration));
        }
    }

    let mut bins: Vec<(usize, u16)> = vec![];
    let i: usize = 0;
    for operating_theater in &instance.operating_theaters {
        bins.push((i, operating_theater.availability[day]));
    }

    biggest_in_biggest_bin_pack(&mut items, &mut bins)
}

//Will be attempted to be parallelized
fn biggest_in_biggest_bin_pack(items: &mut Vec<(usize, u16)>, bins: &mut Vec<(usize, u16)>) -> Result<Vec<Vec<usize>>, String> {
    let mut bin_assignment: Vec<Vec<usize>> = vec![vec![]; bins.len()];

    let f = |a: &(usize, u16), b: &(usize, u16)| (b.1).cmp(&(a.1));

    items.sort_by(f);
    bins.sort_by(f);

    for item in items {
        let mut flag = false;
        'inner: for bin in &mut *bins {
            if item.1 <= bin.1 {
                bin_assignment[bin.0].push(item.0);
                bin.1 -= item.1;
                flag = true;
                break 'inner;
            }
        }
        if !flag {
            return Err("Packing unsuccessful".into());
        }
    }
    Ok(bin_assignment)
}


#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn check_prelim_day_assignment() {
        //load instance and pass to prelim_day_assignment
        let result =
        builder::instance_build(r#"C:\Users\chenv\ihtc2024chen\public_datasets\i01.json"#);

        let Ok(instance) = result else {
            panic!("{}", result.err().unwrap());
        };

        let patients_per_day_per_surgeon = prelim_day_assignment(&instance);
        
        //Check prelim_day_assignment
        let num_days = patients_per_day_per_surgeon[0].len();
        for surgeon_idx in 0..patients_per_day_per_surgeon.len() {
            assert!(patients_per_day_per_surgeon[surgeon_idx].len() == num_days, "num_days is different for different surgeons");
            for day in 0..num_days {
                //checking assignment correctness
                for (first_patient_idx, second_patient_idx) in 
                    (patients_per_day_per_surgeon[surgeon_idx][day]).iter().tuple_windows() {
                    let first_patient_ref = &instance.patients[*first_patient_idx];
                    let second_patient_ref = &instance.patients[*second_patient_idx];
                    
                    assert_eq!(first_patient_ref.surgeon_id, instance.surgeons[surgeon_idx].id, 
                    "patients assigned to wrong surgeons");
                    assert_eq!(first_patient_ref.surgery_release_day, day, 
                    "patient assigned to day other than surgery_release_day");
                    assert!(second_patient_ref.surgery_due_day > first_patient_ref.surgery_due_day || 
                        ((second_patient_ref.surgery_due_day == first_patient_ref.surgery_due_day) && 
                        (second_patient_ref.surgery_duration <= first_patient_ref.surgery_duration)),
                    "increasing ordering patients within day did not work");
                }
            }
        }

    }

    #[test]
    fn check_arrange_patients_for_surgeons() {
        //load instance and pass to prelim_day_assignment
        let result =
        builder::instance_build(r#"C:\Users\chenv\ihtc2024chen\public_datasets\i01.json"#);

        let Ok(instance) = result else {
            panic!("{}", result.err().unwrap());
        };
        
        let mut patients_per_day_per_surgeon = prelim_day_assignment(&instance);

        let result: Result<Vec<VecDeque<usize>>, String> = arrange_patients_for_surgeons(&instance, &mut patients_per_day_per_surgeon, None);
            let Ok(unassigned_per_surgeon) = result else {
                panic!("{}", result.err().unwrap())
            };
            
        for surgeon_idx in 0..instance.surgeons.len() {
            println!("arranging for surgeon {:?}", surgeon_idx);
            let surgeon_id = &instance.surgeons[surgeon_idx].id;
            assert!(unassigned_per_surgeon[surgeon_idx].len() <= 3, "more than 4 unassigned for surgeon {}", surgeon_id);

            // let num_days = patients_per_day_per_surgeon[0].len();
            for (day, day_deque) in (patients_per_day_per_surgeon[surgeon_idx]).iter().enumerate() {
                // assert!(Iterator::sum::<u16>(day_deque.iter().map(|x| instance.patients[*x].surgery_duration)) 
                //     <= instance.surgeons[surgeon_idx].max_surgery_time[day], 
                // "surgeon duration exceeded");

                let mut patient_duration_sum: u16 = 0;
                for patient_idx in day_deque {
                    let patient_ref = &instance.patients[*patient_idx];
                    assert_eq!(&patient_ref.surgeon_id, surgeon_id);
                    assert!(patient_ref.surgery_release_day <= day, "assigned day is before release day");
                    assert!(patient_ref.surgery_due_day >= day, "assigned day is after due day");
                    patient_duration_sum += patient_ref.surgery_duration;
                }
                assert!(patient_duration_sum <= instance.surgeons[surgeon_idx].max_surgery_time[day], 
                "surgeon {} max_surgery_time exceeded", surgeon_id);
            }
        }
        

    }
}