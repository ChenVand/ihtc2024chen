const NUM_DAYS: usize = 28;

// use std::collections::VecDeque;

struct Patient {
    release_date_: u8,
    due_date_: Option<usize>,
    surgeon_: u8,
    assigned_day: u8
}

struct Surgeon {
    capacity_per_day: [usize; NUM_DAYS],
    // every day has a Vec of patient indices, referring to patient_vec
    assigned_per_day: [Vec<usize>; NUM_DAYS],
}

impl Surgeon {
    fn has_valid_schedule(&self) -> Result<(), usize> {
        for day in 0..NUM_DAYS {
            if self.assigned_per_day[day].len() 
                > self.capacity_per_day[day] {
                return Err(day)
            }
        }
        Ok(())
    }

    // Add optionality to select particular patient or day
}

fn dynamic_arrange_patients_for_surgeon(capacity: &[usize], first_day: usize, mut assignment: Vec<Vec<usize>>, patient_vec: &Vec<Patient>) 
    -> Option<Vec<Vec<usize>>> {
    // for now does not change patient_vec
    // change to mutable slice for assignment, instead of ownerships

    assert!(first_day < capacity.len());
    assert!((capacity.len() - first_day) == assignment.len(), "dynamic function received capacity and assignment Vecs of different length");

    for day in first_day..(capacity.len()-1) {
        if assignment[day].len() > capacity[day] {
            for j in 0..assignment[day].len() {
                if let Some(due_day) = patient_vec[assignment[day][j]].due_date_ {
                    if due_day > day {

                        // prepare for recursive func
                        let mut sub_assignment = assignment.split_off(day+1);
                        sub_assignment[0].push(assignment[day][j]);

                        //call recursive func
                        let result =  dynamic_arrange_patients_for_surgeon(capacity, 
                            day+1, sub_assignment.clone(), patient_vec);
                        
                        match result {
                            Some(mut new_sub_assignment) => {
                                assignment[day].remove(j);
                                assignment.append(&mut new_sub_assignment);
                                return Some(assignment);
                            },
                            None => {
                                sub_assignment[0].pop();
                                assignment.append(&mut sub_assignment);
                            }
                        }
                    } else {
                        continue
                    }
                } else {
                    // prepare for recursive func
                    let mut sub_assignment = assignment.split_off(day+1);
                    sub_assignment[0].push(assignment[day][j]);

                    //call recursive func
                    let result =  dynamic_arrange_patients_for_surgeon(capacity, 
                        day+1, sub_assignment.clone(), patient_vec);
                    
                    match result {
                        Some(mut new_sub_assignment) => {
                            assignment[day].remove(j);
                            assignment.append(&mut new_sub_assignment);
                            return Some(assignment);
                        },
                        None => {
                            sub_assignment[0].pop();
                            assignment.append(&mut sub_assignment);
                        }
                    }
                }
            }
            return None
        }
    }
    Some(assignment)
}

fn percolate_patients_for_surgeon(surgeon: &mut Surgeon, patient_vec: &mut Vec<Patient>) {
    // collect initial index from has_valid_schedule(&self) to improve perf

    for day in 0..NUM_DAYS {
        if surgeon.assigned_per_day[day].len() > surgeon.capacity_per_day[day] {    
            for pj in 0..surgeon.assigned_per_day[day].len() {
                match patient_vec[surgeon.assigned_per_day[day][pj]].due_date_ {
                    None => todo!(), //then move pj from surgeon.assigned_per_day[day] to ...[day + 1] and end loop
                    Some(due_day) => todo!() //if due_day>day, move pj as above, otherwise continue
                }
            }
            todo!(); // if reached end of for loop, then no guy can be moved. If this ever happens, I need to change algo!!

            // old code
            // // push_back(&mut self, value: T)
            // // pop_front(&mut self) -> Option<T>
            // let popped_patient_idx = assigned_per_day[day].pop_front().unwrap();

        }
    }
}

// get patients from Json file.
// Initial assignment assigned_day=release_date for each patient
// place patients in decreasing order of permitted stay in Surgeon.assigned_per_day
// so that first out has more flexibility
fn collect_patients() -> (Vec<Patient>, Vec<Surgeon>) {
    let surgeon_vec: Vec<Surgeon> = Vec::new();
    let patient_vec: Vec<Patient> = Vec::new();

    (patient_vec, surgeon_vec)
}

// surgeon num_patients_vec


fn main() {

    // Collect patients:
    // make sruct instance for each patient, 
    // and keep track of number of surgeons and surgeon vec

    let (patient_vec, surgeon_vec) = collect_patients();

    
}
