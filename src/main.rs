mod termplete;

fn add(args: Vec<&str>) ->bool{
    let sum: i32 = args.iter().map(|x| x.parse::<i32>().unwrap_or(0)).sum();
    println!("The sum is {}", sum);
    true
}
use std::collections::HashMap;

use rust_xlsxwriter::*;
use serde_json::Value;

fn write_json_to_excel(json_value: Value) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let mut worksheet = workbook.add_worksheet();

    if let Value::Array(array) = json_value {
        // Write the header (keys of the first object)
        if let Some(Value::Object(first_obj)) = array.get(0) {
            let mut col = 0;
            for key in first_obj.keys() {
                worksheet.write(0, col, key)?;
                col += 1;
            }
        }

        // Write the data
        let mut row = 1;
        for item in array.iter() {
            if let Value::Object(obj) = item {
                let mut col = 0;
                for value in obj.values() {
                    match value {
                        Value::String(s) => {worksheet.write(row, col, s)?;},
                        Value::Number(n) => {
                            if let Some(num) = n.as_i64() {
                                worksheet.write(row, col, num as i32)?;
                            } else if let Some(num) = n.as_f64() {
                                worksheet.write(row, col, num)?;
                            }
                        },
                        Value::Bool(b) => {worksheet.write(row, col, *b)?;},
                        _ => {worksheet.write(row, col, &value.to_string())?;}, // Other types (including nested objects) written as raw string
                    }
                    col += 1;
                }
                row += 1;
            }
        }
    }

    workbook.save("output.xlsx")?;

    Ok(())
}

fn malopin() -> Result<(), XlsxError> {
    let json_data = r#"
        [
            { "name": "John", "age": 30, "city": "New York", "details": { "foo": "bar" } },
            { "name": "Jane", "age": 25, "city": "Chicago", "details": { "bar": "baz" } }
        ]
    "#;
    

    write_json_to_excel(serde_json::from_str(json_data).expect("Failed to parse JSON"))?;

    Ok(())
}



fn main(){
    // HashMap<String, Box<dyn Fn(Vec<&str>) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>>>
    let mut my_map: HashMap<String, Box<dyn Fn(Vec<&str>)->bool>> = HashMap::new();
    my_map.insert("add".to_string(), Box::new(add));
    my_map.insert("multiply".to_string(), Box::new(|args| {
        let product: i32 = args.iter().map(|x| x.parse::<i32>().unwrap_or(0)).product();
        println!("The product is {}", product);
        true
    }));
    my_map.insert("excel".to_string(), Box::new(|_args| {
        malopin();
        true
    }));

    termplete::replloop(my_map);

}