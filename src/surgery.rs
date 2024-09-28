use crate::builder::Instance;
use crate::builder::Patient;
use std::collections::VecDeque;
// use std::iter::Iterator;

// creates assignment of patients per day per surgeon. usize refers to index in 
// instance.patients
// ##patients ordering is in increasing *distance* to due date, and then increasing surgery_duration
fn prelim_day_assignment(instance: &Instance) -> Vec<Vec<VecDeque<usize>>> {
    // output: patient_day_per_surgeon
    todo!();
}

fn arrange_patients_for_surgeon(surgeon_idx: usize, instance: &Instance, patients_per_day_per_surgeon: &mut Vec<Vec<VecDeque<usize>>>) -> Result<(), String> {
    let result = dynamic_arrange_patients_for_surgeon(&instance.surgeons[surgeon_idx].max_surgery_time, &mut patients_per_day_per_surgeon[surgeon_idx], 
        0, &instance.patients)?;
    Ok(result)
}

// ##### input should be surgeon idx, patient_day_per_surgeon, surgeon_capacity_vec, patients (or just instance for previous two), first_day
fn dynamic_arrange_patients_for_surgeon(capacity: &Vec<u16>, assignment: &mut Vec<VecDeque<usize>>, first_day: usize, patients: &Vec<Patient>) 
    -> Result<(), String> {
    
    assert!(first_day < capacity.len());

    for day in first_day..(capacity.len()-1) {
        /*
        for start point:
            while duration exceeds capacity, successively accumulate patients to bump.
            bump these patients and apply function recursively
            if this did not succeed, undo bump and continue to next iteration. 
        */
        let mut start_p: usize = assignment[day].len();
        loop {
            let mut demanded_duration = Iterator::sum::<u16>(assignment[day].iter().map(|x| patients[*x].surgery_duration));
            let mut patients_to_bump: VecDeque<usize> = VecDeque::new();

            // collect patients to be bumped
            'inner: for patient_counter in (start_p-1)..=0 {
            // while demanded_duration > capacity[day] {
                if demanded_duration > capacity[day] {
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
                return Err("Capacity in day {day:?} cannot be reached".into());
            }

            // bump patients (bumped_patient is actually patient index in instance.patients)
            // patient_local_idx decreases
            for patient_local_idx in patients_to_bump.clone() {
                let bumped_patient = assignment[day].remove(patient_local_idx).unwrap();
                assignment[day + 1].push_front(bumped_patient);
            }

            // try recursion, and if it fails, undo bumping to allow for next attempt with lower start_p
            let result = dynamic_arrange_patients_for_surgeon(capacity, assignment, day + 1, patients);
            if result.is_ok() {
                // note that this means that the outermost day loop stops, and so it only goes up to a problematic day.
                return Ok(());
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
        }

    }
    return Ok(());  
        
}

fn bump_patient(patient_idx: usize) -> Result<(),()> {todo!();}

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

//##### change output to Result<Vec<VecDeque<usize>>, String>?
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