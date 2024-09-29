use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};
use serde_json;
// use serde_json::{to_string_pretty, from_str};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
// #[serde(rename_all = "snake_case")]
pub struct Instance {
    pub days: usize,
    // skill_levels: u8,
    // shift_types: Vec<String>,
    // age_groups
    // weights
    // occupants
    pub patients: Vec<Patient>,
    pub surgeons: Vec<Surgeon>,
    pub operating_theaters: Vec<OperatingTheater>,
    //rooms
    //nurses
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Patient {
    pub id: String,
    pub mandatory: bool,
    pub surgery_release_day: usize,
    #[serde(default = "default_due_day")]
    pub surgery_due_day: usize,
    pub surgery_duration: u16,
    pub surgeon_id: String
}

fn default_due_day() -> usize {
    usize::MAX
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Surgeon {
    pub id: String,
    pub max_surgery_time: Vec<u16>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct OperatingTheater {
    id: String,
    pub availability: Vec<u16>,
}

fn deserialize(data: &str) -> Result<Instance, serde_json::Error> {

    let data_struct: Instance = serde_json::from_str(data)?;

    // println!("{:#?}", data_struct);    

    Ok(data_struct)
}

fn instance_build(path: &str) -> Result<Instance, serde_json::Error> {
    let mut file = File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    deserialize(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_instance_build() {
        let result = 
            instance_build(r#"C:\Users\chenv\ihtc2024chen\public_datasets\i01.json"#);
        match result {
            Ok(_) => (),
            Err(error) => {panic!("{}", error);}
        }
    }

    #[test]
    fn check_deserialize() {

        let data = r#"{
            "days": 14,
            "patients": [
                {
                "id": "p00",
                "mandatory": false,
                "gender": "A",
                "age_group": "elderly",
                "length_of_stay": 8,
                "surgery_release_day": 3,
                "surgery_duration": 120,
                "surgeon_id": "s0",
                "incompatible_room_ids": []
                },
                {
                "id": "p01",
                "mandatory": false,
                "gender": "B",
                "age_group": "elderly",
                "length_of_stay": 2,
                "surgery_release_day": 1,
                "surgery_duration": 90,
                "surgeon_id": "s0",
                "incompatible_room_ids": []
                }
            ],
            "surgeons": [
                {
                "id": "s0",
                "max_surgery_time": [
                    0,
                    480,
                    360,
                    480,
                    480,
                    0,
                    0,
                    360,
                    0,
                    0,
                    480,
                    600,
                    0,
                    480
                ]
                }
            ],
            "operating_theaters": [
                {
                "id": "t0",
                "availability": [
                    480,
                    720,
                    720,
                    720,
                    600,
                    600,
                    720,
                    720,
                    720,
                    720,
                    720,
                    600,
                    600,
                    720
                ]
                },
                {
                "id": "t1",
                "availability": [
                    480,
                    600,
                    600,
                    600,
                    720,
                    600,
                    0,
                    600,
                    600,
                    720,
                    600,
                    480,
                    600,
                    0
                ]
                }
            ]
        }"#;

        let result = deserialize(data);
        match result {
            Ok(_) => (),
            Err(error) => {panic!("{}", error);}
        }
        // assert_eq!(tickets, tickets2);
    }

    #[test]
    fn check_deserialize_2() {

        let data = r#"{
            "days": 14,
            "patients": [
                {
                "id": "p00",
                "mandatory": false,
                "gender": "A",
                "age_group": "elderly",
                "length_of_stay": 8,
                "surgery_release_day": 3,
                "surgery_duration": 120,
                "surgeon_id": "s0",
                "incompatible_room_ids": []
                }
            ],
            "surgeons": [
            ],
            "operating_theaters": [
                {
                "id": "t1",
                "availability": [
                    480,
                    600
                ]
                }
            ]
        }"#;

        let desired_data_struct = Instance{
            days: 14, 
            patients: vec![Patient{
                id: "p00".into(),
                mandatory: false,
                surgery_release_day: 3,
                surgery_due_day: usize::MAX,
                surgery_duration: 120,
                surgeon_id: "s0".into()
            }],
            surgeons: vec![],
            operating_theaters: vec![OperatingTheater{
                id: "t1".into(),
                availability: vec![480, 600]
            }]
        };

        let result = deserialize(data);
        match result {
            Ok(data_struct) => {
                assert_eq!(data_struct, desired_data_struct);
            },
            Err(error) => {panic!("{}", error);}
        }
    }
}


/*extern crate rustc_serialize;
use rustc_serialize::json::Json;
use std::fs::File;
use std::io::Read;

fn main() {
    let mut file = File::open("text.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let json = Json::from_str(&data).unwrap();
    println!("{}", json.find_path(&["Address", "Street"]).unwrap());
} */