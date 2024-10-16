use crate::builder::*;
use std::collections::{HashMap, VecDeque, BTreeMap};

pub struct Assignment<'a> {
    pub instance: &'a Instance,
    in_progress: bool,
    patients_per_day_per_surgeon: Vec<Vec<VecDeque<usize>>>,
}

impl<'a> Assignment<'a> {

    //##### Change this to return dict according to surgeon idx
    pub fn weighted_patient_by_surgeon_for_day (&self, day: usize) -> Vec<((usize, u16), VecDeque<(usize, u16)>)> {
        let weighted_patients_per_surgeon: Vec<VecDeque<(usize, u16)>> = self.patients_per_day_per_surgeon.iter()
        .map(|surgeon_vec| surgeon_vec[day].iter().map(
        |&idx| (idx, self.instance.patients[idx].surgery_duration)).collect())
        .collect();

        // each surgeon is named and weighted: vec[i]: ((surgeon_idx, tot_duration), VecDeque<(patient_idx, duration)>)
        weighted_patients_per_surgeon.into_iter()
            .enumerate()
            .map(|(surgeon_idx, patients)| ((surgeon_idx, patients.iter().map(|x| x.1).sum::<u16>()), patients))
            .collect()
    }
}