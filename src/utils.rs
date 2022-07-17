use lazy_static::lazy_static;
use regex::Regex;

use crate::composegenerator::v4::types::Permissions;

lazy_static! {
    static ref SYNTAX1_REGEX: Regex = Regex::new(r"\$\{.*?}").unwrap();
    static ref SYNTAX2_REGEX: Regex = Regex::new(r"\$[A-z1-9]+").unwrap();
}

pub fn find_env_vars(string: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let found_things = SYNTAX1_REGEX.captures_iter(string);
    let found_things2 = SYNTAX2_REGEX.captures_iter(string);
    for captures in found_things {
        for element in captures.iter().flatten() {
            let matched = element.as_str();
            // Remove the leading "${" and trailing "}"
            let var_name = &matched[2..matched.len() - 1];
            result.push(var_name.to_string());
        }
    }
    for captures in found_things2 {
        for element in captures.iter().flatten() {
            let matched = element.as_str();
            // Remove the $
            let var_name = &matched[1..matched.len()];
            result.push(var_name.to_string());
        }
    }
    result
}

#[cfg(test)]
mod test {
    use crate::utils::find_env_vars;

    #[test]
    fn handle_empty_properly() {
        let result = find_env_vars(&"Example value 123$ test".to_string());
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn find_syntax_1() {
        let result = find_env_vars(&"something${BITCOIN_IP}something".to_string());
        assert_eq!(result, vec!["BITCOIN_IP".to_string()]);
    }

    #[test]
    fn find_syntax_2() {
        let result = find_env_vars(&"something $BITCOIN_IP something".to_string());
        assert_eq!(result, vec!["BITCOIN_IP".to_string()]);
    }

    #[test]
    fn find_syntax_combined() {
        let result =
            find_env_vars(&"something $BITCOIN_IP something ${LND_IP} $ANOTHER_THING".to_string());
        let expected = vec![
            "BITCOIN_IP".to_string(),
            "LND_IP".to_string(),
            "ANOTHER_THING".to_string(),
        ];

        assert!(expected.iter().all(|item| result.contains(item)));
    }
}

pub fn flatten(perms: Vec<Permissions>) -> Vec<String> {
    let mut result = Vec::<String>::new();
    for perm in perms {
        match perm {
            Permissions::OneDependency(dependency) => {
                result.push(dependency);
            }
            Permissions::AlternativeDependency(mut dependencies) => {
                result.append(&mut dependencies);
            }
        }
    }
    result
}
