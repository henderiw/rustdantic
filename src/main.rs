//use std::default;
//use std::collections::{BTreeMap, HashMap};
use serde::{Deserialize, Serialize};
use serde_json;
use default_derive::Default as ChoreoDefault;
use validate_derive::Validate as ChoreoValidate;
use choreo_derive::ChoreoResource;
use ::choreo_api::{Validate, Defaultable};
use ::choreo_core::Resource;
//use serde_with;

/* 
#[derive(ChoreoDefault, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

#[derive(ChoreoDefault, ChoreoValidate, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MyStruct {
    //#[validate("required, minLength=5")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cdefault("mieke")]
    #[cvalidate("required")]
    #[cvalidate("fn=validate_name")]
    #[cvalidate("minLength=3")]
    #[cvalidate("pattern=^[a-zA-Z]+$")]
    pub name: Option<String>,
    #[cvalidate("mo=20")]
    #[cvalidate("required")]
    #[cdefault(20)]
    pub age: Option<u32>,
    pub nested: NestedStruct,
    pub option_nested: Option<NestedStruct>,
    #[cvalidate("maxItems=1")]
    pub vec_nested: Vec<NestedStruct>,
    #[cvalidate("maxItems=1")]
    pub option_vec_nested: Option<Vec<NestedStruct>>,
    #[cvalidate("minItems=2")]
    pub hashmap_nested: HashMap<String, NestedStruct>,
    #[cvalidate("minItems=2")]
    pub option_hashmap_nested: Option<HashMap<String, NestedStruct>>,
    #[cvalidate("minItems=2")]
    pub btreemap_nested: BTreeMap<String, NestedStruct>,

    pub option_btreemap_nested: Option<BTreeMap<String, NestedStruct>>,

    #[cdefault("enum=Active")]
    pub option_enum: Option<Status>,
}

impl MyStruct {
    pub fn validate_name(&self) -> Result<(), String> {
        if let Some(ref name) = self.name {
            if name.len() < 3 {
                return Err(format!(
                    "Name '{}' is too short. Must be at least 3 characters.",
                    name
                ));
            }
        }
        Ok(())
    }
}

#[derive(ChoreoDefault, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NestedStruct {
    //#[cvalidate("required, minLength=5")]
    //#[cvalidate("fn=validate_fn")]
    #[cdefault("mieke")]
    pub n: Option<String>,
    //#[cvalidate("ge=20")]
    pub i: Option<u32>,
}

impl NestedStruct {
    pub fn validate_fn(&self) -> Result<(), String> {
        if let Some(n) = &self.n {
            if n.starts_with("W") {
                return Err(format!("Value '{}' must start with 'W'", n));
            }
        }
        Ok(())
    }
}
*/

/// Spec object for Dummy
#[derive(ChoreoResource, ChoreoDefault, ChoreoValidate, Deserialize, Serialize, Clone, Debug, Default)]
#[choreo(
    group = "example.com",
    version = "v1alpha1",
    kind = "Dummy",
    status_name = "DummyStatus",
    derive = "Default"
)]
pub struct DummySpec {
    //#[serde(skip_serializing_if = "Option::is_none")]
    #[cvalidate("le=10")]
    #[cdefault(20)]
    val: Option<u32>,
}

#[derive(ChoreoDefault, ChoreoValidate, Deserialize, Serialize, Clone, Debug, Default)]
pub struct DummyStatus {
    //#[serde(flatten)]
    condition_status: ::choreo_meta::ConditionStatus,
}

fn main() {
    let mut dummy_spec = DummySpec::default();
    dummy_spec.apply_defaults();
    let d = Dummy::new("wim", dummy_spec);
    let api_version = Dummy::api_version(&());
    println!("api_version {}", api_version);
    match serde_json::to_string_pretty(&d) {
        Ok(json) => println!("Serialized JSON:\n{}", json),
        Err(e) => println!("Failed to serialize to JSON: {}", e),
    }

    let api_version = Dummy::api_version(&());
    println!("api_version {}", api_version);

    let meta = d.meta();
    println!("meta {:?}", meta);

    println!("value {:?}", d.spec.val);
    match d.validate() {
        Ok(_) => println!("## Validation passed"),
        Err(err) => println!("## Validation failed: \n{}", err),
    }

    let json_input = r#"
    {
      "apiVersion": "example.com/v1alpha1",
      "kind": "Dummy",
      "metadata": {
        "name": "wim"
      },
      "spec": {
        "val": 5
      }
    }
    "#;
    // Deserialize the JSON into the Dummy struct
    let dummy = match serde_json::from_str::<Dummy>(json_input) {
        Ok(deserialized) => {
            println!("Deserialized struct:\n{:?}", deserialized);
            deserialized
        }
        Err(e) => {
            println!("Failed to deserialize JSON: {}", e);
            return;
        }
    };
    println!("value {:?}", dummy.spec.val);
    match dummy.validate() {
        Ok(_) => println!("## Validation passed"),
        Err(err) => println!("## Validation failed: \n{}", err),
    }
    /*
    match dummy.spec.validate() {
        Ok(_) => println!("Validation passed"),
        Err(err) => println!("Validation failed: \n{}", err),
    }
    */

    /* 
    let mut my_struct = MyStruct {
        //name: Some("wim".to_string()),
        name: None,
        age: None,

        nested: NestedStruct {
            n: None,
            //n: Some("Wim".to_string()),
            //name: None,
            i: Some(10),
        },

        option_nested: Some(NestedStruct { n: None, i: None }),

        option_vec_nested: Some(vec![NestedStruct { n: None, i: None }]),

        vec_nested: vec![NestedStruct { n: None, i: None }],

        hashmap_nested: {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), NestedStruct { n: None, i: None });
            map
        },
        option_hashmap_nested: {
            Some({
                let mut map = HashMap::new();
                map.insert("key1".to_string(), NestedStruct { n: None, i: None });
                map
            })
        },
        btreemap_nested: {
            let mut map = BTreeMap::new();
            map.insert("key1".to_string(), NestedStruct { n: None, i: None });
            map
        },
        option_btreemap_nested: {
            Some({
                let mut map = BTreeMap::new();
                map.insert("key1".to_string(), NestedStruct { n: None, i: None });
                map
            })
        },
        option_enum: None,
    };
    println!("my_struct before default {:?}", &my_struct);
    my_struct.set_defaults();
    println!("my_struct after default {:?}", &my_struct);
    match my_struct.validate() {
        Ok(_) => println!("validation succeeded:\n"),
        Err(e) => println!("validation failed: {}", e),
    }

    println!("my_struct {:?}", &my_struct);

    match serde_json::to_string_pretty(&my_struct) {
        Ok(json) => println!("Serialized JSON:\n{}", json),
        Err(e) => println!("Failed to serialize to JSON: {}", e),
    }

    */
    
}
