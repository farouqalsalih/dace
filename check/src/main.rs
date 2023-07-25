use csv::Reader;
use std::error::Error;
use std::str::FromStr;
use stack_alg_sim::olken::LRUSplay;
use hist::Hist;
use std::collections::HashMap;
use std::sync::Arc;
use std::{env};


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let trace = &args[1];
    let program_name = format!("../../locapo/locapo/npc_version/{}.csv", trace);
    println!("{}", program_name);
    let mut reader = Reader::from_path(program_name)?;

    let mut analyzer = LRUSplay::<usize>::new();
    let mut histogram = Hist::new();

    for result in reader.records() {
        let record = result?;
        let number_str = &record[0];
        match usize::from_str(number_str) {
            Ok(number) => {
                let dist = analyzer.access(number);
                if dist == Some(100) {
                    println!("{}", dist.unwrap());
                }
                histogram.add_dist(dist);

            },
            Err(_) => eprintln!("Could not parse number: {}", number_str),
        };
    }
    
    let bucket = "cpp-data-comparison";

    let hist_rd_path_json = Arc::new(format!(
        "json/hist/rd/{}_Olken.json",
        *trace
    ));
    let hist_rd_path_csv = Arc::new(format!(
        "csv/hist/rd/{}_Olken.csv",
        *trace
    ));

    let hist_vec = histogram.to_vec();

    let serialized_hist_rd_data = Arc::new(
        serde_json::to_string(
            &hist_vec
                .into_iter()
                .map(|(k, v)| {
                    (
                        match k {
                            Some(key) => key.to_string(),
                            None => String::from("None"),
                        },
                        v,
                    )
                })
                .collect::<HashMap<String, usize>>(),
        )
        .expect("Failed to serialize"),
    );

    let handle1 = tokio::spawn({
        let serialized_hist_rd_data = Arc::clone(&serialized_hist_rd_data);
        let hist_rd_path_json = Arc::clone(&hist_rd_path_json);
        async move {
            let serialized_hist_rd_data = &serialized_hist_rd_data;
            let hist_rd_path_json = &hist_rd_path_json;
            aws_utilities::s3::save_serialized(
                serialized_hist_rd_data,
                bucket,
                hist_rd_path_json,
            )
            .await
        }
    });

    let handle2 = tokio::spawn({
        let hist_rd_path_csv = Arc::clone(&hist_rd_path_csv);
        async move { aws_utilities::s3::save_csv_hist(histogram, bucket, &hist_rd_path_csv).await }
    });

    // Store all handles in a Vec
    let handles = vec![handle1, handle2];

    // Await them all
    for handle in handles {
        handle.await??; // Use '?' if the functions return Result<_, _>
    }

    Ok(())
}
