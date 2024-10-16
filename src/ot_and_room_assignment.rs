use std::collections::{HashMap, VecDeque, BTreeMap};
use itertools::Itertools;
use minilp::{Variable, Problem, OptimizationDirection, ComparisonOp};

use crate::surgery_assignment;
use crate::builder::{Instance, Patient};
use crate::assignment::Assignment;

/*Assign OT and room day by day, and if infeasible then call surgery_assignment::bump_patient
to re-assign from that day forward while keeping previous days intact.
To do this, need to chose patient to bump. Recommended patient should have small duration, and long time to due date.*/

// pub fn assign_ots_and_rooms(instance: &Instance, day_assignment: &mut DayAssignment) {todo!();}

impl<'a> Assignment<'a> {

    fn patient_OT_assignment_for_day(&mut self, day: usize) {
        /*Returns patients-by-OT assignment (according to ot index in theaters).  */  
        // outer Vec corresponds to OTs as ordered in instance.operating_theaters
    
        let instance = self.instance;
        let num_bins = instance.theaters.len();
        let num_surgeons = instance.surgeons.len();

        // each surgeon is named and weighted: vec[i]: ((surgeon_idx, tot_duration), VecDeque<(patient_idx, duration)>)
        let surgeon_durations_partition_map = self.get_surgeon_durations_partition_map(day);

        // Set up bin_gradient
        struct Bin {
            theater_idx: usize,
            capacity: u16,
            gradient_weight: f64
        }
        let mut bins: BTreeMap<usize, Bin> = BTreeMap::new();
        let mut bin_capacity_vec = instance.theaters.iter().enumerate()
        .map(|(idx, theater)| (idx, theater.availability[day])).collect_vec();
        bin_capacity_vec.sort_by(|a,b| b.1.cmp(&a.1));       
        bin_capacity_vec.into_iter().enumerate()
        .map(|(order, (idx, capacity))| {
            bins.insert(idx, Bin{theater_idx: idx, capacity, gradient_weight: order as f64})
        }).collect_vec();
        let bin_gradient_weight = |bin_idx: usize| {bins.get(&bin_idx).unwrap().gradient_weight};
    
        // Setting up LP   
        let mut problem = Problem::new(OptimizationDirection::Minimize);
    
        //Surgeon variables X_{surgeon,bin}^(i,j) where j<=i
        let i_max = 1;
        let mut x_map: BTreeMap<(usize, usize, u8, u8), Variable> = BTreeMap::new();
        for surgeon_idx in 0..num_surgeons {
            for bin_idx in 0..num_bins {
                for i in 0..=i_max {
                    for j in 0..=i {
                        if j==0 {
                            x_map.insert((surgeon_idx, bin_idx, i, j), 
                                problem.add_var(i as f64 * instance.weights.surgeon_transfer + bin_gradient_weight(bin_idx),
                                (0.0, 1.0)));
                        } else {
                            x_map.insert((surgeon_idx, bin_idx, i, j), 
                                problem.add_var(bin_gradient_weight(bin_idx), (0.0, 1.0)));                
                        }
                    }
                }
            }
        }
           
        //Bin variables Y_bin
        let mut y_map: BTreeMap<usize, Variable> = BTreeMap::new();
        for bin_idx in 0..num_bins {
            y_map.insert(bin_idx, problem.add_var(instance.weights.open_operating_theater, (0.0, 1.0)));
        }
    
        // Constraint X^(i,0)=X^(i,1)=...=X^(i,i)
        for surgeon_idx in 0..num_surgeons {
            for i in 0..=i_max {
                for j in 0..i {
                    let mut summands = (0..num_bins)
                        .map(|bin_idx| (*x_map.get(&(surgeon_idx, bin_idx, i, j)).unwrap(), 1.0))
                        .collect_vec();
                    let mut negative_summands = (0..num_bins)
                    .map(|bin_idx| (*x_map.get(&(surgeon_idx, bin_idx, i, j+1)).unwrap(), -1.0))
                    .collect_vec();
                    summands.append(&mut negative_summands);
                    problem.add_constraint(summands, ComparisonOp::Eq, 0.0);
                }
            }
        }

        // Constraint \sum_i{X^(i,0)} = 1
        for surgeon_idx in 0..num_surgeons {
            let mut summands = Vec::with_capacity(i_max as usize * num_bins);
            for i in 0..=i_max {
                for bin_idx in 0..num_bins {
                    summands.push((*x_map.get(&(surgeon_idx, bin_idx, i, 0)).unwrap(), 1.0));
                }
            }
            problem.add_constraint(summands, ComparisonOp::Eq, 1.0);
        }

        //Constraint \sum_{surgeon,i,j} {duration(s,i,j) * X_{surgeon, bin}^(i,j)} <= capacity(bin) * Y_bin
        for bin_idx in 0..num_bins {
            let mut summands = Vec::with_capacity((((i_max+1) * (i_max+2))/2) as usize * num_surgeons + 1);
            for surgeon_idx in 0..num_surgeons {
                for i in 0..=i_max {
                    for j in 0..=i {
                        let duration: f64 = if i==0 { 
                            surgeon_durations_partition_map.get(&surgeon_idx).unwrap().total_duration as f64
                        } else if j==0 {
                            surgeon_durations_partition_map.get(&surgeon_idx).unwrap().partitioned_durations.0 as f64
                        } else if j==1 {
                            surgeon_durations_partition_map.get(&surgeon_idx).unwrap().partitioned_durations.1 as f64
                        } else {
                            panic!("should not be able to reach here.");
                        };
                        
                        summands.push((*x_map.get(&(surgeon_idx, bin_idx, i, j)).unwrap(), -duration));
                    }
                }
            }
            summands.push((*y_map.get(&bin_idx).unwrap(), instance.theaters[bin_idx].availability[day] as f64));
            problem.add_constraint(summands, ComparisonOp::Ge, 0.0);
        }

        //##### solve LP

        //##### perform rounding

        //##### if didn't work, return error so that patient will be bumped

    }
}


//biggest_in_biggest_bin_pack
/*
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
*/