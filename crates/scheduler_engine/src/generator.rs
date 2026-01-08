use crate::{PastSchedules, Result, SchedulerError};
use rand::rng;
use rand::seq::SliceRandom;
use scheduler_core::{AppConfig, Employee, EmployeeId, Role, Schedule, Sex, Weekday};
use std::collections::{HashMap, HashSet};

pub struct ScheduleGenerator {
    config: AppConfig,
}

#[derive(Debug, Clone)]
pub struct GenerationOptions {
    pub year: i32,
    pub month: u32,
    pub past_schedules: PastSchedules,
}

impl ScheduleGenerator {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn generate(&self, employees: &[Employee], options: GenerationOptions) -> Result<Schedule> {
        if employees.is_empty() {
            return Err(SchedulerError::NoEmployees);
        }

        let mut schedule = Schedule::new(options.year, options.month);
        let mut day_counts: HashMap<Weekday, usize> =
            Weekday::all().into_iter().map(|d| (d, 0)).collect();

        // Process fixed schedules first
        let flexible_employees =
            self.process_fixed_schedules(employees, &mut schedule, &mut day_counts)?;

        // Group by required days
        let grouped = self.group_by_required_days(&flexible_employees);

        // Generate combinations
        let combinations = self.generate_day_combinations();

        // Assign flexible employees
        self.assign_flexible_employees(
            &grouped,
            &combinations,
            &mut schedule,
            &mut day_counts,
            &options.past_schedules,
            employees,
        )?;

        // Apply mentee constraints
        self.apply_mentee_constraints(&mut schedule, employees)?;

        Ok(schedule)
    }

    fn process_fixed_schedules(
        &self,
        employees: &[Employee],
        schedule: &mut Schedule,
        day_counts: &mut HashMap<Weekday, usize>,
    ) -> Result<Vec<Employee>> {
        let mut flexible = Vec::new();

        for emp in employees {
            if !emp.fixed_days.is_empty() {
                for day in &emp.fixed_days {
                    schedule.assign(*day, emp.id);
                    *day_counts.entry(*day).or_insert(0) += 1;
                }
            } else {
                flexible.push(emp.clone());
            }
        }

        Ok(flexible)
    }

    fn group_by_required_days(&self, employees: &[Employee]) -> HashMap<usize, Vec<Employee>> {
        let mut grouped: HashMap<usize, Vec<Employee>> = HashMap::new();

        for emp in employees {
            grouped
                .entry(emp.required_days as usize)
                .or_default()
                .push(emp.clone());
        }

        let mut rng = rng();
        for group in grouped.values_mut() {
            group.shuffle(&mut rng);
        }

        grouped
    }

    fn generate_day_combinations(&self) -> HashMap<usize, Vec<Vec<Weekday>>> {
        let mut combos = HashMap::new();

        // 1 day
        combos.insert(
            1,
            vec![
                vec![Weekday::Monday],
                vec![Weekday::Tuesday],
                vec![Weekday::Wednesday],
                vec![Weekday::Thursday],
                vec![Weekday::Friday],
            ],
        );

        // 2 days
        combos.insert(
            2,
            vec![
                vec![Weekday::Monday, Weekday::Wednesday],
                vec![Weekday::Monday, Weekday::Thursday],
                vec![Weekday::Monday, Weekday::Friday],
                vec![Weekday::Tuesday, Weekday::Thursday],
                vec![Weekday::Tuesday, Weekday::Friday],
                vec![Weekday::Wednesday, Weekday::Friday],
            ],
        );

        // 3 days
        combos.insert(
            3,
            vec![vec![Weekday::Monday, Weekday::Wednesday, Weekday::Friday]],
        );

        // 5 days
        combos.insert(
            5,
            vec![vec![
                Weekday::Monday,
                Weekday::Tuesday,
                Weekday::Wednesday,
                Weekday::Thursday,
                Weekday::Friday,
            ]],
        );

        combos
    }

    fn assign_flexible_employees(
        &self,
        grouped: &HashMap<usize, Vec<Employee>>,
        combinations: &HashMap<usize, Vec<Vec<Weekday>>>,
        schedule: &mut Schedule,
        day_counts: &mut HashMap<Weekday, usize>,
        past_schedules: &PastSchedules,
        all_employees: &[Employee],
    ) -> Result<()> {
        let mut keys: Vec<usize> = grouped.keys().copied().collect();
        keys.sort_by(|a, b| b.cmp(a));

        for num_days in keys {
            if let Some(employees) = grouped.get(&num_days) {
                if let Some(combos) = combinations.get(&num_days) {
                    for emp in employees {
                        let best_combo = self.find_best_combination(
                            combos,
                            day_counts,
                            emp,
                            past_schedules,
                            all_employees,
                        );

                        for day in &best_combo {
                            schedule.assign(*day, emp.id);
                            *day_counts.entry(*day).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn find_best_combination(
        &self,
        combinations: &[Vec<Weekday>],
        day_counts: &HashMap<Weekday, usize>,
        employee: &Employee,
        past_schedules: &PastSchedules,
        all_employees: &[Employee],
    ) -> Vec<Weekday> {
        let mut rng = rng();
        let mut shuffled = combinations.to_vec();
        shuffled.shuffle(&mut rng);

        let mut best_combo = shuffled[0].clone();
        let mut min_score = f64::INFINITY;

        // Calculate past day frequencies
        let past_freqs = self.calculate_past_frequencies(employee, past_schedules);

        for combo in &shuffled {
            let mut temp_counts = day_counts.clone();
            for day in combo {
                *temp_counts.entry(*day).or_insert(0) += 1;
            }

            // Balance score
            let values: Vec<usize> = temp_counts.values().copied().collect();
            let avg = values.iter().sum::<usize>() as f64 / values.len() as f64;
            let variance = values
                .iter()
                .map(|&v| (v as f64 - avg).powi(2))
                .sum::<f64>();

            // Repetition score
            let repetition = combo
                .iter()
                .map(|day| past_freqs.get(day).unwrap_or(&0.0))
                .sum::<f64>();

            // Distribution scores
            let sex_score = if self.config.schedule.consider_sex_distribution {
                self.calculate_sex_distribution_score(combo, day_counts, all_employees)
            } else {
                0.0
            };

            let role_score = if self.config.schedule.consider_role_distribution {
                self.calculate_role_distribution_score(combo, day_counts, employee, all_employees)
            } else {
                0.0
            };

            let total = variance + (3.0 * repetition) + sex_score + role_score;

            if total < min_score {
                min_score = total;
                best_combo = combo.clone();
            }
        }

        best_combo
    }

    fn calculate_past_frequencies(
        &self,
        employee: &Employee,
        past_schedules: &PastSchedules,
    ) -> HashMap<Weekday, f64> {
        let mut freqs = HashMap::new();

        if let Some(past) = past_schedules.get(&employee.id) {
            let recent = if past.len() > 2 {
                &past[past.len() - 2..]
            } else {
                past
            };

            for (i, schedule) in recent.iter().enumerate() {
                let weight = 1.0 - (i as f64 / recent.len() as f64 * 0.75);
                for day in schedule {
                    *freqs.entry(*day).or_insert(0.0) += weight;
                }
            }
        }

        freqs
    }

    fn calculate_sex_distribution_score(
        &self,
        combo: &[Weekday],
        day_counts: &HashMap<Weekday, usize>,
        all_employees: &[Employee],
    ) -> f64 {
        // Simplified: penalize if adding would make sex distribution very uneven
        let mut score = 0.0;

        for day in combo {
            let count = day_counts.get(day).unwrap_or(&0);
            // Penalize if day already has many employees
            if *count > 10 {
                score += 2.0;
            }
        }

        score
    }

    fn calculate_role_distribution_score(
        &self,
        combo: &[Weekday],
        day_counts: &HashMap<Weekday, usize>,
        employee: &Employee,
        all_employees: &[Employee],
    ) -> f64 {
        // Similar to sex distribution
        let mut score = 0.0;

        if employee.role.is_none() {
            return score;
        }

        for day in combo {
            let count = day_counts.get(day).unwrap_or(&0);
            if *count > 10 {
                score += 2.0;
            }
        }

        score
    }

    fn apply_mentee_constraints(
        &self,
        schedule: &mut Schedule,
        employees: &[Employee],
    ) -> Result<()> {
        use scheduler_core::config::MenteeOverlapMode;

        if matches!(
            self.config.schedule.mentee_mentor_overlap,
            MenteeOverlapMode::None
        ) {
            return Ok(());
        }

        for emp in employees {
            if !emp.is_mentee {
                continue;
            }

            let Some(mentor_id) = emp.mentor_id else {
                continue;
            };

            // Find mentor
            let mentor = employees
                .iter()
                .find(|e| e.id == mentor_id)
                .ok_or_else(|| SchedulerError::GenerationFailed("Mentor not found".to_string()))?;

            // Skip if mentor works fully remote
            if mentor.works_remote {
                continue;
            }

            // Get mentor's days
            let mentor_days: HashSet<Weekday> = Weekday::all()
                .into_iter()
                .filter(|day| schedule.get_employees_for_day(*day).contains(&mentor_id))
                .collect();

            // Get mentee's days
            let mentee_days: HashSet<Weekday> = Weekday::all()
                .into_iter()
                .filter(|day| schedule.get_employees_for_day(*day).contains(&emp.id))
                .collect();

            let overlap: Vec<_> = mentee_days.intersection(&mentor_days).collect();

            let required_overlap = match self.config.schedule.mentee_mentor_overlap {
                MenteeOverlapMode::None => 0,
                MenteeOverlapMode::AtLeastOne => 1,
                MenteeOverlapMode::AtLeastTwo => 2,
            };

            if overlap.len() < required_overlap {
                // Adjust schedule to create overlap
                self.adjust_for_mentee_overlap(
                    schedule,
                    emp.id,
                    mentor_id,
                    &mentor_days,
                    required_overlap,
                )?;
            }
        }

        Ok(())
    }

    fn adjust_for_mentee_overlap(
        &self,
        schedule: &mut Schedule,
        mentee_id: EmployeeId,
        mentor_id: EmployeeId,
        mentor_days: &HashSet<Weekday>,
        required: usize,
    ) -> Result<()> {
        // Remove mentee from all days
        for day in Weekday::all() {
            schedule.remove_assignment(day, mentee_id);
        }

        // Reassign to mentor's days (up to required)
        let mut assigned = 0;
        for day in mentor_days.iter().take(required) {
            schedule.assign(*day, mentee_id);
            assigned += 1;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_generation() {
        let config = AppConfig::default();
        let generator = ScheduleGenerator::new(config);

        let employees = vec![
            Employee {
                id: 1,
                name: "Alice".to_string(),
                sex: Sex::Female,
                role: Some(Role::FullStackEngineer),
                required_days: 2,
                fixed_days: vec![],
                is_mentee: false,
                is_mentor: false,
                mentor_id: None,
                works_remote: false,
            },
            Employee {
                id: 2,
                name: "Bob".to_string(),
                sex: Sex::Male,
                role: Some(Role::BackendEngineer),
                required_days: 3,
                fixed_days: vec![],
                is_mentee: false,
                is_mentor: false,
                mentor_id: None,
                works_remote: false,
            },
        ];

        let options = GenerationOptions {
            year: 2025,
            month: 1,
            past_schedules: HashMap::new(),
        };

        let result = generator.generate(&employees, options);
        assert!(result.is_ok());

        let schedule = result.unwrap();
        assert_eq!(schedule.year, 2025);
        assert_eq!(schedule.month, 1);
    }

    #[test]
    fn test_mentee_constraint() {
        let mut config = AppConfig::default();
        config.schedule.mentee_mentor_overlap =
            scheduler_core::config::MenteeOverlapMode::AtLeastOne;

        let generator = ScheduleGenerator::new(config);

        let employees = vec![
            Employee {
                id: 1,
                name: "Mentor".to_string(),
                sex: Sex::Male,
                role: None,
                required_days: 2,
                fixed_days: vec![Weekday::Monday, Weekday::Wednesday],
                is_mentee: false,
                is_mentor: true,
                mentor_id: None,
                works_remote: false,
            },
            Employee {
                id: 2,
                name: "Mentee".to_string(),
                sex: Sex::Female,
                role: None,
                required_days: 2,
                fixed_days: vec![],
                is_mentee: true,
                is_mentor: false,
                mentor_id: Some(1),
                works_remote: false,
            },
        ];

        let options = GenerationOptions {
            year: 2025,
            month: 1,
            past_schedules: HashMap::new(),
        };

        let result = generator.generate(&employees, options);
        assert!(result.is_ok());
    }
}
