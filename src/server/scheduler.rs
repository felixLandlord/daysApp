use crate::server::schema::{
    DayCombination, DayCount, Employee, MonthlySchedule, PastSchedules, ScheduleGenerator, Weekday,
}; // ScheduleStatistics
use rand::{rng, seq::SliceRandom};
use std::collections::HashMap;

pub fn generate_schedule(
    generator: &ScheduleGenerator,
    employees: &[Employee],
    past_schedules: &PastSchedules,
) -> MonthlySchedule {
    // let mut rng = rng();
    let mut day_counts: DayCount = generator
        .weekdays
        .iter()
        .map(|day| (day.clone(), 0))
        .collect();

    let mut schedule: MonthlySchedule = generator
        .weekdays
        .iter()
        .map(|day| (day.clone(), Vec::new()))
        .collect();

    // Process employees with fixed schedules first
    let (flexible_employees, fixed_employees) =
        process_fixed_schedules(employees, &mut day_counts, &mut schedule);

    // Group flexible employees by required days
    let grouped_employees = group_by_required_days(&flexible_employees);

    // Process flexible employees (prioritize those with more required days)
    process_flexible_employees(
        generator,
        grouped_employees,
        &mut day_counts,
        &mut schedule,
        past_schedules,
    );

    schedule
}

fn process_fixed_schedules(
    employees: &[Employee],
    day_counts: &mut DayCount,
    schedule: &mut MonthlySchedule,
) -> (Vec<Employee>, Vec<Employee>) {
    let mut flexible_employees = Vec::new();
    let mut fixed_employees = Vec::new();

    for employee in employees {
        if !employee.fixed_days.is_empty() {
            // This employee has fixed days
            for day in &employee.fixed_days {
                if let Some(daily_schedule) = schedule.get_mut(day) {
                    if !daily_schedule.iter().any(|e| e.id == employee.id) {
                        daily_schedule.push(employee.clone());
                        *day_counts.entry(day.clone()).or_insert(0) += 1;
                    }
                }
            }
            fixed_employees.push(employee.clone());
        } else {
            flexible_employees.push(employee.clone());
        }
    }

    (flexible_employees, fixed_employees)
}

fn group_by_required_days(employees: &[Employee]) -> HashMap<usize, Vec<Employee>> {
    let mut grouped: HashMap<usize, Vec<Employee>> = HashMap::new();

    for employee in employees {
        grouped
            .entry(employee.required_days as usize)
            .or_insert_with(Vec::new)
            .push(employee.clone());
    }

    // Shuffle each group for randomization
    let mut rng = rng();
    for (_days, group) in grouped.iter_mut() {
        group.shuffle(&mut rng);
    }

    grouped
}

fn process_flexible_employees(
    generator: &ScheduleGenerator,
    grouped_employees: HashMap<usize, Vec<Employee>>,
    day_counts: &mut DayCount,
    schedule: &mut MonthlySchedule,
    past_schedules: &PastSchedules,
) {
    // Sort keys by number of required days (higher first)
    let mut keys: Vec<usize> = grouped_employees.keys().cloned().collect();
    keys.sort_by(|a, b| b.cmp(a));

    for num_days in keys {
        if let Some(employees_list) = grouped_employees.get(&num_days) {
            if let Some(available_combos) = generator.day_combinations.get(&num_days) {
                for employee in employees_list {
                    // Find best day combination
                    let best_combo = find_best_day_combination(
                        available_combos,
                        day_counts,
                        employee,
                        past_schedules,
                    );

                    // Assign employee to days from the best combination
                    for day in &best_combo.days {
                        if let Some(daily_schedule) = schedule.get_mut(day) {
                            if !daily_schedule.iter().any(|e| e.id == employee.id) {
                                daily_schedule.push(employee.clone());
                                *day_counts.entry(day.clone()).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn find_best_day_combination(
    available_combos: &[DayCombination],
    day_counts: &DayCount,
    employee: &Employee,
    past_schedules: &PastSchedules,
) -> DayCombination {
    let mut rng = rng();
    let mut shuffled_combos = available_combos.to_vec();
    shuffled_combos.shuffle(&mut rng);

    let mut best_combo = shuffled_combos[0].clone();
    let mut min_score = f64::INFINITY;

    // Calculate past day frequencies with recency weighting
    let mut past_day_frequencies: HashMap<Weekday, f64> = HashMap::new();

    // Set lookback limit
    let lookback_limit = 2;

    // Calculate day frequencies from past schedules
    if let Some(past_employee_schedules) = past_schedules.get(&employee.id) {
        let recent_schedules = if past_employee_schedules.len() > lookback_limit {
            &past_employee_schedules[past_employee_schedules.len() - lookback_limit..]
        } else {
            past_employee_schedules
        };

        for (i, past_schedule) in recent_schedules.iter().enumerate() {
            // More recent schedules have higher weight
            let recency_weight = 1.0 - (i as f64 / recent_schedules.len() as f64 * 0.75);

            for day in past_schedule {
                *past_day_frequencies.entry(day.clone()).or_insert(0.0) += recency_weight;
            }
        }
    }

    for combo in &shuffled_combos {
        // Create temp counts to evaluate this combination
        let mut temp_counts = day_counts.clone();
        for day in &combo.days {
            *temp_counts.entry(day.clone()).or_insert(0) += 1;
        }

        // Calculate variance as measure of balance
        let values: Vec<usize> = temp_counts.values().cloned().collect();
        let avg_count = values.iter().sum::<usize>() as f64 / values.len() as f64;
        let variance = values
            .iter()
            .map(|&count| (count as f64 - avg_count).powi(2))
            .sum::<f64>();

        // Calculate repetition score
        let repetition_score = combo
            .days
            .iter()
            .map(|day| past_day_frequencies.get(day).unwrap_or(&0.0))
            .sum::<f64>();

        // Combined score
        let repetition_weight = 3.0;
        let total_score = variance + (repetition_weight * repetition_score);

        if total_score < min_score {
            min_score = total_score;
            best_combo = combo.clone();
        }
    }

    best_combo
}

// pub fn generate_statistics(
//     weekdays: &Vec<Weekday>,
//     schedule: &MonthlySchedule,
//     employees: &[Employee],
// ) -> ScheduleStatistics {
//     let mut day_counts: HashMap<Weekday, usize> = HashMap::new();
//     let mut gender_distribution: HashMap<Weekday, HashMap<String, usize>> = HashMap::new();
//     let mut role_distribution: HashMap<Weekday, HashMap<String, usize>> = HashMap::new();

//     // Initialize statistics data structures
//     for day in weekdays {
//         day_counts.insert(day.clone(), 0);
//         gender_distribution.insert(day.clone(), HashMap::new());
//         role_distribution.insert(day.clone(), HashMap::new());
//     }

//     // Process schedule to compute statistics
//     for (day, employees_list) in schedule {
//         day_counts.insert(day.clone(), employees_list.len());

//         for employee in employees_list {
//             // Gender stats
//             *gender_distribution
//                 .entry(day.clone())
//                 .or_default()
//                 .entry(employee.sex.to_string())
//                 .or_insert(0) += 1;

//             // Role stats
//             *role_distribution
//                 .entry(day.clone())
//                 .or_default()
//                 .entry(employee.role.to_string())
//                 .or_insert(0) += 1;
//         }
//     }

//     let total_employees = employees.len();
//     let total_days = weekdays.len();

//     let total_attendances: usize = day_counts.values().sum();
//     let average_daily_attendance = if total_days > 0 {
//         total_attendances as f64 / total_days as f64
//     } else {
//         0.0
//     };

//     ScheduleStatistics {
//         day_counts,
//         gender_distribution,
//         role_distribution,
//         total_employees,
//         average_daily_attendance,
//     }
// }

// Main function to generate balanced office schedules
pub fn generate_balanced_schedule(
    employees: &[Employee],
    past_schedules: &PastSchedules,
) -> MonthlySchedule {
    // return value (MonthlySchedule, ScheduleStatistics)
    let generator = ScheduleGenerator::new();
    let schedule = generate_schedule(&generator, employees, past_schedules);

    // let statistics = generate_statistics(&generator.weekdays, &schedule, employees);

    // (schedule, statistics)
    schedule
}
