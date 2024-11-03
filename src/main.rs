#[derive(Debug, PartialEq, Clone)]
struct Tree {
    value: f32,
    leaves: Option<(Box<Tree>, Box<Tree>)>,
    probability: Option<f32>,
}

impl Tree {
    fn display(&self, level: usize) {
        let indent = "    ".repeat(level);
        print!("{}Node(value: {:.2}; ", indent, self.value);

        if let Some(prob) = self.probability {
            print!("probability: {:.2})", prob);
        } else {
            print!("probability: None)");
        }

        println!(); // Newline after displaying the node

        if let Some((ref left, ref right)) = self.leaves {
            left.display(level + 1);
            right.display(level + 1);
        }
    }
}


fn get_users_f32(prompt: &str, min: Option<f32>) -> Result<f32, &'static str> {
    loop {
        println!("{prompt}");

        let mut user_input = String::new();
        std::io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");


        match (user_input.trim().parse::<f32>(), min) {
            (Ok(val), None) => return Ok(val),
            (Ok(val), Some(min)) if val >= min => return Ok(val),
            _ => {
                println!("{} is not a valid digit", user_input.trim());
                continue;
            }
        };
    }
}

fn get_users_y_or_n(prompt: &str) -> Result<bool, &'static str> {
    loop {
        println!("{prompt}");

        let mut user_input = String::new();
        std::io::stdin()
            .read_line(&mut user_input)
            .expect("extend_or_not: Failed to read line");

        match user_input.trim() {
            "y" => return Ok(true),
            "n" => return Ok(false),
            _ => { println!("Wrong input {}; Only \"y\" OR \"n\" are allowed.", user_input.trim()) }
        }
    };
}


fn get_starting_node() -> Result<Tree, &'static str> {
    Ok(Tree {
        value: get_users_f32("Enter value for the starting node!", None)?,
        leaves: None,
        probability: Some(1.0),
    })
}


fn extend_tree(tree_to_extend: &mut Tree) -> Result<(), &'static str> {
    let parent_val = tree_to_extend.value;

    match get_users_y_or_n(&format!("Do you want to extend the {parent_val} node further?"))? {
        false => return Ok(()),
        true => {}
    }

    let (val0, val1) = (
        get_users_f32(&format!("Enter value for the first child of node = {}", parent_val), None)?,
        get_users_f32(&format!("Enter value for the second child of node = {}", parent_val), None)?
    );

    let mut new_leaves = (
        Box::new(Tree { value: val0, leaves: None, probability: None }),
        Box::new(Tree { value: val1, leaves: None, probability: None })
    );

    extend_tree(&mut new_leaves.0)?;
    extend_tree(&mut new_leaves.1)?;

    tree_to_extend.leaves = Some(new_leaves);
    Ok(())
}


fn find_risk_neutral_q(tree: &mut Tree, r: f32) -> Result<(), String> {
    if let Some((ref mut t_1, ref mut t_2)) = tree.leaves {
        let parent_probability = tree.probability.unwrap();

        let q = (tree.value * (1.0 + r) - t_2.value) / (t_1.value - t_2.value);

        match q {
            0.0..1.0 => {
                t_1.probability = Some(parent_probability * q);
                find_risk_neutral_q(t_1, r)?;

                t_2.probability = Some(parent_probability * (1.0 - q));
                find_risk_neutral_q(t_2, r)?;
            }
            _ => return Err(format!("\n\nRisk neutral Q does not exist for parent={} -> child_1={}; child_2={}; discount factor {r}\n\n", tree.value, t_1.value, t_2.value))
        }
    }
    Ok(())
}


fn find_pv(tree: &Tree, strike: f32, is_call: bool, r: f32, mut depth: i32) -> f32 {
    if let Some((ref t0, ref t1)) = tree.leaves {
        depth += 1;
        find_pv(t0, strike, is_call, r, depth) + find_pv(t1, strike, is_call, r, depth)
    } else {
        match is_call {
            true => tree.probability.unwrap() * (0.0_f32.max(tree.value - strike) / (1.0 + r).powi(depth)),
            false => tree.probability.unwrap() * (0.0_f32.max(strike - tree.value) / (1.0 + r).powi(depth))
        }
    }
}


fn main() {
    loop {
        println!("\n\n\nConstruction of the tree has started!");
        let mut tree = match get_starting_node() {
            Ok(node) => node,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };

        extend_tree(&mut tree).unwrap();
        println!("\n\nThe tree you have constructed:\n");
        tree.display(0);

        let r = match get_users_f32("\n\nEnter discounting factor (r). Should be >= 0", Some(0.0)) {
            Ok(r) => r,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };

        match find_risk_neutral_q(&mut tree, r) {
            Ok(_) => {
                println!("\n\nThe tree with risk neutral probabilities:");
                tree.display(0);
            }
            Err(e) => {
                println!("{e}");
                continue;
            }
        };


        let is_call = match get_users_y_or_n("\n\nIs the option Call or Put? (y for Call, n for Put)") {
            Ok(is_call) => is_call,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };

        let strike = match get_users_f32("\n\nEnter the strike price for the option", None) {
            Ok(strike) => strike,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };


        println!("\n\n");
        tree.display(0);
        println!("Fair price of Option is_call={is_call} with strike={strike}; r={r} is:\n{}", find_pv(&tree, strike, is_call, r, 0));
    }
}
