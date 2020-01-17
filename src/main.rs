use std::env;
use std::process;
use std::fs::File;
use std::io::{self, Read, Write, BufReader, BufWriter};

use log4rs;
use log::{info, error, debug};
use byteorder::{BigEndian, ReadBytesExt};

fn create_logger(filename: &str) {
    let file_logger = log4rs::append::file::FileAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new("{d} {l} - {m}{n}")))
        .build(filename).unwrap();

    let config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build("file_logger", Box::new(file_logger)))
        .build(log4rs::config::Root::builder().appender("file_logger").build(log::LevelFilter::Debug))
        .unwrap();

    let _log_handle = log4rs::init_config(config).unwrap();
}

fn convert_files<R: Read>(number_of_nodes: u32, mut temperature_field_sub: R, mut time_temperature_history: R, mut velocity_info: R) -> io::Result<()> {
    // Read in temperature field sub time step
    let num_of_sub_steps = temperature_field_sub.read_u32::<BigEndian>()?;
    debug!("num_of_sub_steps: {}", num_of_sub_steps);
    let current_step = temperature_field_sub.read_u32::<BigEndian>()?;
    debug!("current_step: {}", current_step);

    for sub_step1 in 1..(num_of_sub_steps + 1) {
        let dt = temperature_field_sub.read_f64::<BigEndian>()?;
        let sub_step2 = temperature_field_sub.read_u32::<BigEndian>()?;
        let time_value = temperature_field_sub.read_f64::<BigEndian>()?;
        if sub_step1 != sub_step2 {
            error!("Number of sub steps do not match: {} != {}", sub_step1, sub_step2);
            break
        }
        for i in 1..(number_of_nodes + 1) {
            let n_id = temperature_field_sub.read_u32::<BigEndian>()?;
            let px = temperature_field_sub.read_f64::<BigEndian>()?;
            let py = temperature_field_sub.read_f64::<BigEndian>()?;
            let pz = temperature_field_sub.read_f64::<BigEndian>()?;
            let temp = temperature_field_sub.read_f64::<BigEndian>()?;
            if i != n_id {
                error!("Node id does not match: {} != {}", i, n_id);
                break
            }
            // TODO: write output to file
        }
    }


    // Read in time temperature history
    let num_of_points = time_temperature_history.read_u32::<BigEndian>()?;
    debug!("num_of_points: {}", num_of_points);
    let current_step = time_temperature_history.read_u32::<BigEndian>()?;
    debug!("current_step: {}", current_step);
    let ntime = time_temperature_history.read_u32::<BigEndian>()?;
    debug!("ntime: {}", ntime);

    for sub_step1 in 1..(ntime + 1) {
        let sub_step2 = time_temperature_history.read_u32::<BigEndian>()?;
        let time_history_value1 = time_temperature_history.read_f64::<BigEndian>()?;
        if sub_step1 != sub_step2 {
            error!("Number of sub steps do not match: {} != {}", sub_step1, sub_step2);
            break
        }
    }

    for sub_step1 in 1..(ntime + 1) {
        for id1 in 1..(num_of_points + 1) {
            let id2 = time_temperature_history.read_u32::<BigEndian>()?;
            let temperature = time_temperature_history.read_f64::<BigEndian>()?;
            if id1 != id2 {
                error!("Node id does not match: {} != {}", id1, id2);
                break
            }
            let px = time_temperature_history.read_f64::<BigEndian>()?;
            let py = time_temperature_history.read_f64::<BigEndian>()?;
            let pz = time_temperature_history.read_f64::<BigEndian>()?;
            let vx = time_temperature_history.read_f64::<BigEndian>()?;
            let vy = time_temperature_history.read_f64::<BigEndian>()?;
            let vz = time_temperature_history.read_f64::<BigEndian>()?;
        }
    }


    // Read in velocity info
    let start_step = velocity_info.read_u32::<BigEndian>()?;
    debug!("start_step: {}", start_step);
    let number_of_surface_nodes = velocity_info.read_u32::<BigEndian>()?;
    debug!("number_of_surface_nodes: {}", number_of_surface_nodes);

    for current_step1 in (0..start_step).rev() {
        let current_step2 = velocity_info.read_u32::<BigEndian>()?;
        if current_step1 != current_step2 {
            error!("Steps to not match: {} != {}", current_step1, current_step2)
        }
        for current_node1 in 1..(number_of_surface_nodes + 1) {
            let current_node2 = velocity_info.read_u32::<BigEndian>()?;
            if current_node1 != current_node2 {
                error!("Node id does not match: {} != {}", current_node1, current_node2);
                break
            }
            let px = velocity_info.read_f64::<BigEndian>()?;
            let py = velocity_info.read_f64::<BigEndian>()?;
            let pz = velocity_info.read_f64::<BigEndian>()?;
            let vx = velocity_info.read_f64::<BigEndian>()?;
            let vy = velocity_info.read_f64::<BigEndian>()?;
            let vz = velocity_info.read_f64::<BigEndian>()?;
        }
    }

    Ok(())
}

fn process_files(output_path: &str, number_of_nodes: u32) -> io::Result<()> {
    // Files to read in:
    // temperature_field_sub_0001.bin
    // time_temperature_history_0001.bin
    // velocity_info_0001.bin

    for i in 1.. {
        let name_temperature_field_sub = format!("{}/temperature_field_sub_{:04}", output_path, i);
        let name_time_temperature_history = format!("{}/time_temperature_history_{:04}", output_path, i);
        let name_velocity_info = format!("{}/velocity_info_{:04}", output_path, i);


        let f = File::open(name_temperature_field_sub)?;
        let file_temperature_field_sub = BufReader::new(f);


        let f = File::open(name_time_temperature_history)?;
        let file_time_temperature_history = BufReader::new(f);


        let f = File::open(name_velocity_info)?;
        let file_velocity_info = BufReader::new(f);

        convert_files(number_of_nodes, file_temperature_field_sub, file_time_temperature_history, file_velocity_info);
    }

    Ok(())
}

fn main() {
    create_logger("pecube_conv.log");

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        error!("You have to specify the path to the Pecube output files.");
        error!("And the number of nodes:");
        info!("{} output/ 100", args[0]);
        process::exit(1);
    }

    let output_path = &args[1];
    let number_of_nodes: u32 = args[2].parse().unwrap();

    match process_files(output_path, number_of_nodes) {
        Ok(_) => {
            info!("Processing finished");
        }
        Err(e) => {
            error!("An error occured: {}", e);
        }
    };
}
