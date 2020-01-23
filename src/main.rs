use std::env;
use std::process;
use std::fs::File;
use std::io::{self, Write, BufReader, BufWriter};
use std::fmt;

use log4rs;
use log::{info, error, debug};
use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
enum ConvertError {
    IoError(io::Error),
    NodeID,
    SubStep,
    Step,
    FileStep,
}

impl From<io::Error> for ConvertError {
    fn from(error: io::Error) -> Self {
        ConvertError::IoError(error)
    }
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConvertError::IoError(e) =>  write!(f, "{}", e),
            ConvertError::NodeID => write!(f, "The node ID does not match"),
            ConvertError::SubStep => write!(f, "The sub step index does not match"),
            ConvertError::Step => write!(f, "The step index does not match"),
            ConvertError::FileStep => write!(f, "The file step index does not match"),
        }
    }
}

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

fn convert_files(name_temperature_field_sub: &str, name_time_temperature_history: &str,
        name_velocity_info: &str, current_file_step: u32) -> Result<(), ConvertError> {
    let name_in_temperature_field_sub = format!("{}.bin", name_temperature_field_sub);
    let name_out_temperature_field_sub = format!("{}.txt", name_temperature_field_sub);
    debug!("Open input file: {}", name_in_temperature_field_sub);
    let f = File::open(name_in_temperature_field_sub)?;
    let mut in_temperature_field_sub = BufReader::new(f);
    debug!("Open output file: {}", name_out_temperature_field_sub);
    let f = File::create(name_out_temperature_field_sub)?;
    let mut out_temperature_field_sub = BufWriter::new(f);

    let name_in_time_temperature_history = format!("{}.bin", name_time_temperature_history);
    let name_out_time_temperature_history = format!("{}.txt", name_time_temperature_history);
    debug!("Open input file: {}", name_in_time_temperature_history);
    let f = File::open(name_in_time_temperature_history)?;
    let mut in_time_temperature_history = BufReader::new(f);
    debug!("Open output file: {}", name_out_time_temperature_history);
    let f = File::create(name_out_time_temperature_history)?;
    let mut out_time_temperature_history = BufWriter::new(f);

    let name_in_velocity_info = format!("{}.bin", name_velocity_info);
    let name_out_velocity_info = format!("{}.txt", name_velocity_info);
    debug!("Open input file: {}", name_in_velocity_info);
    let f = File::open(name_in_velocity_info)?;
    let mut in_velocity_info = BufReader::new(f);
    debug!("Open output file: {}", name_out_velocity_info);
    let f = File::create(name_out_velocity_info)?;
    let mut out_velocity_info = BufWriter::new(f);


    // Read in temperature field sub time step
    let num_of_sub_steps = in_temperature_field_sub.read_u32::<LittleEndian>()?;
    debug!("temperature_field_sub: num_of_sub_steps: {}", num_of_sub_steps);
    let current_step = in_temperature_field_sub.read_u32::<LittleEndian>()?;
    debug!("current_step: {}", current_step);
    let num_of_points = in_temperature_field_sub.read_u32::<LittleEndian>()?;
    debug!("num_of_points (model): {}", num_of_points);
    // let num_of_points = 1400;

    writeln!(out_temperature_field_sub, "# current_step, sub_step, dt, time, node_id, px, py, pz, temperature")?;

    for sub_step1 in 1..(num_of_sub_steps + 1) {
        debug!("sub_step1: {}", sub_step1);
        let dt = in_temperature_field_sub.read_f64::<LittleEndian>()?;
        debug!("    dt: {}", dt);
        let sub_step2 = in_temperature_field_sub.read_u32::<LittleEndian>()?;
        debug!("    sub_step2: {}", sub_step2);
        let time_value = in_temperature_field_sub.read_f64::<LittleEndian>()?;
        debug!("    time_value: {}", time_value);
        if sub_step1 != sub_step2 {
            error!("    Number of sub steps do not match: {} != {}", sub_step1, sub_step2);
            return Err(ConvertError::SubStep)
        }
        for i in 1..(num_of_points + 1) {
            let n_id = in_temperature_field_sub.read_u32::<LittleEndian>()?;
            let px = in_temperature_field_sub.read_f64::<LittleEndian>()?;
            let py = in_temperature_field_sub.read_f64::<LittleEndian>()?;
            let pz = in_temperature_field_sub.read_f64::<LittleEndian>()?;
            let temp = in_temperature_field_sub.read_f64::<LittleEndian>()?;
            if i != n_id {
                error!("        Node id does not match: {} != {}", i, n_id);
                return Err(ConvertError::NodeID)
            }
            writeln!(out_temperature_field_sub, "{}, {}, {}, {}, {}, {}, {}, {}, {}", current_step,
                sub_step1, dt, time_value, n_id, px, py, pz, temp)?;
        }
    }


    // Read in time temperature history
    let num_of_points = in_time_temperature_history.read_u32::<LittleEndian>()?;
    debug!("num_of_points (surface): {}", num_of_points);
    let outer_step = in_time_temperature_history.read_u32::<LittleEndian>()?;
    debug!("outer_step: {}", outer_step);
    if outer_step != current_file_step {
        error!("Number of file steps do not match: {} != {}", outer_step, current_file_step);
        return Err(ConvertError::FileStep)
    }

    // let outer_step = current_file_step

    writeln!(out_time_temperature_history, "# current_step, ntime, sub_step, time, node_id, temperature, px, py, pz, vx, vy, vz")?;

    for current_step1 in (1..(outer_step + 1)).rev() {
        let current_step2 = in_time_temperature_history.read_u32::<LittleEndian>()?;
        debug!("time_temperature_history: current_step2: {}", current_step2);
        let ntime = in_time_temperature_history.read_u32::<LittleEndian>()?;
        debug!("ntime: {}", ntime);

        if current_step1 != current_step2 {
            error!("Number of steps do not match: {} != {}", current_step1, current_step2);
            return Err(ConvertError::Step)
        }

        let mut time_history_values: Vec<f64> = Vec::new();

        for sub_step1 in 1..(ntime + 1) {
            debug!("sub_step1: {}", sub_step1);
            let sub_step2 = in_time_temperature_history.read_u32::<LittleEndian>()?;
            debug!("    sub_step2: {}", sub_step2);
            time_history_values.push(in_time_temperature_history.read_f64::<LittleEndian>()?);
            debug!("    time_history_values[i]: {}", time_history_values[(sub_step1 - 1) as usize]);
            if sub_step1 != sub_step2 {
                error!("Number of sub steps do not match: {} != {}", sub_step1, sub_step2);
                return Err(ConvertError::SubStep)
            }
        }

        for sub_step1 in 1..(ntime + 1) {
            debug!("sub_step1: {}", sub_step1);
            for id1 in 1..(num_of_points + 1) {
                debug!("id1: {}", id1);
                let id2 = in_time_temperature_history.read_u32::<LittleEndian>()?;
                debug!("    id2: {}", id2);
                let temperature = in_time_temperature_history.read_f64::<LittleEndian>()?;
                debug!("    temperature: {}", temperature);
                if id1 != id2 {
                    error!("        Node id does not match: {} != {}", id1, id2);
                    return Err(ConvertError::NodeID)
                }
                let px = in_time_temperature_history.read_f64::<LittleEndian>()?;
                let py = in_time_temperature_history.read_f64::<LittleEndian>()?;
                let pz = in_time_temperature_history.read_f64::<LittleEndian>()?;
                let vx = in_time_temperature_history.read_f64::<LittleEndian>()?;
                let vy = in_time_temperature_history.read_f64::<LittleEndian>()?;
                let vz = in_time_temperature_history.read_f64::<LittleEndian>()?;
                writeln!(out_time_temperature_history, "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}", current_step1,
                    ntime, sub_step1, time_history_values[(sub_step1 - 1) as usize], id1, temperature, px, py, pz, vx, vy, vz)?;
            }
        }
    }


    // Read in velocity info
    let start_step = in_velocity_info.read_u32::<LittleEndian>()?;
    debug!("velocity_info: start_step: {}", start_step);
    let number_of_surface_nodes = in_velocity_info.read_u32::<LittleEndian>()?;
    debug!("number_of_surface_nodes: {}", number_of_surface_nodes);

    writeln!(out_velocity_info, "# current_step, node_id, px, py, pz, vx, vy, vz")?;

    for current_step1 in (1..(start_step + 1)).rev() {
        debug!("current_step1: {}", current_step1);
        let current_step2 = in_velocity_info.read_u32::<LittleEndian>()?;
        debug!("    current_step2: {}", current_step2);
        if current_step1 != current_step2 {
            error!("    Steps to not match: {} != {}", current_step1, current_step2);
            return Err(ConvertError::SubStep)
        }
        for current_node1 in 1..(number_of_surface_nodes + 1) {
            let current_node2 = in_velocity_info.read_u32::<LittleEndian>()?;
            if current_node1 != current_node2 {
                error!("        Node id does not match: {} != {}", current_node1, current_node2);
                return Err(ConvertError::NodeID)
            }
            let px = in_velocity_info.read_f64::<LittleEndian>()?;
            let py = in_velocity_info.read_f64::<LittleEndian>()?;
            let pz = in_velocity_info.read_f64::<LittleEndian>()?;
            let vx = in_velocity_info.read_f64::<LittleEndian>()?;
            let vy = in_velocity_info.read_f64::<LittleEndian>()?;
            let vz = in_velocity_info.read_f64::<LittleEndian>()?;
            writeln!(out_velocity_info, "{}, {}, {}, {}, {}, {}, {}, {}", current_step1, current_node1, px, py, pz, vx, vy, vz)?;
        }
    }

    Ok(())
}

fn process_files(output_path: &str) -> Result<(), ConvertError> {
    // Files to read in:
    // temperature_field_sub_0001.bin
    // mpi_mc_process_0000/output/temperature_field_sub_0001.bin
    // time_temperature_history_0001.bin
    // velocity_info_0001.bin

    for i in 1.. {
        let name_temperature_field_sub = format!("{}/temperature_field_sub_{:04}", output_path, i);
        let name_time_temperature_history = format!("{}/time_temperature_history_{:04}", output_path, i);
        let name_velocity_info = format!("{}/velocity_info_{:04}", output_path, i);

        convert_files(&name_temperature_field_sub, &name_time_temperature_history, &name_velocity_info, i)?;
    }

    Ok(())
}

fn main() {
    create_logger("pecube_conv.log");

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        error!("You have to specify the path to the Pecube output files:");
        info!("{} output/", args[0]);
        process::exit(1);
    }

    let output_path = &args[1];

    match process_files(output_path) {
        Ok(_) => {
            info!("Processing finished");
        }
        Err(e) => {
            error!("An error occurred: {}", e);
        }
    };
}
