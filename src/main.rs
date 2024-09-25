const NUM_DAYS: usize = 28;

// use std::collections::VecDeque;

struct Patient {
    release_date_: usize,
    due_date_: Option<usize>,
    // surgeon_: u8,
    // assigned_day: u8
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

fn dynamic_arrange_patients_for_surgeon(capacity: &[usize], assignment: &mut Vec<Vec<usize>>, first_day: usize, patient_vec: &Vec<Patient>) 
    -> bool {
    // for now does not change patient_vec
    // change to mutable slice for assignment, instead of ownerships

    assert!(first_day < capacity.len());

    for day in first_day..(capacity.len()-1) {
        let day_assignment_length = assignment[day].len();
        if day_assignment_length > capacity[day] {
            for j in 0..day_assignment_length {
                if let Some(due_day) = patient_vec[assignment[day][j]].due_date_ {
                    if due_day > day {

                        // Bumping patient
                        let bumped_patient = assignment[day].remove(j);
                        assignment[day + 1].push(bumped_patient);

                        if dynamic_arrange_patients_for_surgeon(capacity, assignment, day + 1, patient_vec) {
                            return true
                        } else {
                            //Cancelling the bump
                            let returned_patient = assignment[day+1].pop().expect("day+1 out of bound for assignment");
                            assignment[day].insert(j, returned_patient);
                        }
                    }
                } else {
                    // same code as above

                    // Bumping patient
                    let bumped_patient = assignment[day].remove(j);
                    assignment[day + 1].push(bumped_patient);

                    if dynamic_arrange_patients_for_surgeon(capacity, assignment, day + 1, patient_vec) {
                        return true
                    } else {
                        //Cancelling the bump
                        let returned_patient = assignment[day+1].pop().expect("day+1 out of bound for assignment");
                        assignment[day].insert(j, returned_patient);
                    }
                }
            }
            return false
        }
    }
    true
}


fn main() {

    // Collect patients:
    // make sruct instance for each patient, 
    // and keep track of number of surgeons and surgeon vec

    // let (patient_vec, surgeon_vec) = collect_patients();

    
}
