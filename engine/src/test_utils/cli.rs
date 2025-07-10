use std::{
    collections::HashSet,
    io::{self, Write},
};

pub struct CliChoiceProvider;

impl CliChoiceProvider {
    pub fn select_from_list<T, F>(prompt: &str, items: &[T], display: F) -> usize
    where
        F: Fn(&T) -> String,
    {
        println!("\n{}", prompt);
        for (i, item) in items.iter().enumerate() {
            println!("  [{:>2}] {}", i + 1, display(item));
        }
        Self::read_index(items.len())
    }

    pub fn select_multiple<T, F>(
        prompt: &str,
        items: &[T],
        num_choices: u8,
        display: F,
        unique: bool,
    ) -> Vec<T>
    where
        T: Clone + std::hash::Hash + Eq,
        F: Fn(&T) -> String,
    {
        let mut selected = Vec::new();
        let mut seen = HashSet::new();
        let items_vec: Vec<_> = items.iter().collect();
        println!("\n{}", prompt);
        for (i, item) in items_vec.iter().enumerate() {
            println!("  [{:>2}] {}", i + 1, display(item));
        }

        while selected.len() < num_choices as usize {
            let idx = Self::read_index(items_vec.len());
            let item = items_vec[idx];
            if unique && seen.contains(item) {
                println!("Already selected, please choose a different item.");
                continue;
            }
            selected.push(item.clone());
            if unique {
                seen.insert(item);
            }
            println!(
                "Selected: {} ({}/{})",
                display(item),
                selected.len(),
                num_choices
            );
        }
        selected
    }

    pub fn read_index(max: usize) -> usize {
        loop {
            print!("Enter choice (1-{}): ", max);
            io::stdout().flush().unwrap();

            let mut line = String::new();
            if io::stdin().read_line(&mut line).is_err() {
                println!("Error reading input, try again.");
                continue;
            }

            match line.trim().parse::<usize>() {
                Ok(n) if n >= 1 && n <= max => return n - 1,
                _ => println!("Invalid number, please enter between 1 and {}.", max),
            }
        }
    }
}
