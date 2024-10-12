use std::collections::{HashMap, VecDeque, BTreeMap};
use crate::surgery_assignment;
use crate::surgery_assignment::DayAssignment;
use crate::builder::{Instance, Patient};

/*Assign OT and room day by day, and if infeasible then call surgery_assignment::bump_patient
to re-assign from that day forward while keeping previous days intact.
To do this, need to chose patient to bump. Recommended patient should have small duration, and long time to due date.*/

pub fn assign_ots_and_rooms(instance: &Instance, day_assignment: &mut DayAssignment) {todo!();}

//##### implement patient bumping of doesn't work
fn patient_OT_assignment_for_day(instance: &Instance, patients_per_day_per_surgeon: &mut Vec<Vec<VecDeque<usize>>>, day: usize) -> Result<Vec<Vec<usize>>, String> {
    todo!();
    
    // outer Vec corresponds to OTs as ordered in instance.operating_theaters

    /*let mut surgeon_packed_items: Vec<VecDeque<(usize, u16)>> = vec![];
    for surgeon_vec in & *patients_per_day_per_surgeon {
        for patient_idx in &surgeon_vec[day] {
            surgeon_packed_items.push((*patient_idx, instance.patients[*patient_idx].surgery_duration));
        }
    }

    let mut bins: Vec<(usize, u16)> = vec![];
    let i: usize = 0;
    for operating_theater in &instance.operating_theaters {
        bins.push((i, operating_theater.availability[day]));
    }

    biggest_in_biggest_bin_pack(&mut items, &mut bins)*/

    //##### If bin packer fails, call patient bumper
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