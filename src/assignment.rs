use crate::builder::*;
use std::collections::{VecDeque, BTreeMap};

pub struct Assignment<'a> {
    pub instance: &'a Instance,
    pub in_progress: bool,
    //key: surgeon_idx, value: patient deques per day
    pub patients_per_day_per_surgeon: BTreeMap<usize, Vec<VecDeque<usize>>>,
}

//Implemented only for partitions of size 2
pub struct SurgeonPartitionInfo {
    // pub patient_deq: &'b VecDeque<usize>,
    pub total_duration: u16,
    pub partition_location: usize,
    pub partitioned_durations: (u16, u16),
}

impl<'a> Assignment<'a> {

    //key: surgeon_index, value: (total_duration, deque<(patient_idx, duration)>)
    pub fn get_surgeon_durations_partition_map (&self, day: usize) -> BTreeMap<usize, SurgeonPartitionInfo> {
        let mut surgeon_durations_map = BTreeMap::new();
        let get_partition_idx = |patient_deq: &VecDeque<usize>| {
            patient_deq.len()/2
        };

        for surgeon_idx in 0..self.instance.surgeons.len() {
            let patient_deq: &VecDeque<usize> =  &self.patients_per_day_per_surgeon.get(&surgeon_idx).unwrap()[day];
            let duration_vec: Vec<u16> = patient_deq.iter().map(|&idx| self.instance.patients[idx].surgery_duration).collect();
            let partition_location = get_partition_idx(patient_deq);
            surgeon_durations_map.insert(surgeon_idx, SurgeonPartitionInfo{
                // patient_deq,
                total_duration: duration_vec.iter().sum(),
                partition_location,
                partitioned_durations: {
                    (duration_vec.iter().enumerate().filter(|x| x.0 <= partition_location).map(|x| x.1).sum(),
                    duration_vec.iter().enumerate().filter(|x| x.0 > partition_location).map(|x| x.1).sum())
                }
            });
        }

        surgeon_durations_map

        //trash
        /*
        let weighted_patients_per_surgeon: Vec<VecDeque<(usize, u16)>> = self.patients_per_day_per_surgeon.iter()
        .map(|surgeon_vec| surgeon_vec[day].iter().map(
        |&idx| (idx, self.instance.patients[idx].surgery_duration)).collect())
        .collect();

        // each surgeon is named and weighted: vec[i]: ((surgeon_idx, tot_duration), VecDeque<(patient_idx, duration)>)
        weighted_patients_per_surgeon.into_iter()
            .enumerate()
            .map(|(surgeon_idx, patients)| ((surgeon_idx, patients.iter().map(|x| x.1).sum::<u16>()), patients))
            .collect()
        */
    }
}