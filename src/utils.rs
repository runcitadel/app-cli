use lazy_static::lazy_static;
use regex::Regex;

use crate::composegenerator::types::Permissions;

lazy_static! {
    // This should have been the following regex originally: \$(\{.*?}|[A-z1-9]+)
    // However, it lead to a double match of ${VAR} and {VAR} getting matched for some reason
    static ref ENV_VAR_REGEX: Regex = Regex::new(r"\$\{.*?}|\$[A-z1-9]+").unwrap();
}

#[macro_export]
macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key.to_string(), $value);
            )+
            m
        }
     };
);

#[macro_export]
macro_rules! bmap(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::BTreeMap::new();
            $(
                m.insert($key.to_string(), $value);
            )+
            m
        }
     };
);

pub fn find_env_vars(string: &str) -> Vec<&str> {
    let mut result: Vec<&str> = Vec::new();
    let matches = ENV_VAR_REGEX.captures_iter(string);
    for captures in matches {
        for element in captures.iter().flatten() {
            let matched = element.as_str();
            // If the env var starts with ${, remove it and the closing }
            // Otherwise, just remove the $
            if matched.starts_with("${") {
                result.push(&matched[2..matched.len() - 1])
            } else {
                result.push(&matched[1..matched.len()]);
            };
        }
    }
    result
}

#[cfg(test)]
mod test_env_vars {
    use crate::utils::find_env_vars;

    #[test]
    fn handle_empty_properly() {
        let result = find_env_vars("Example value 123$ test");
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn find_syntax_1() {
        let result = find_env_vars("something${BITCOIN_IP}something");
        assert_eq!(result, vec!["BITCOIN_IP"]);
    }

    #[test]
    fn find_syntax_2() {
        let result = find_env_vars("something $BITCOIN_IP something");
        assert_eq!(result, vec!["BITCOIN_IP"]);
    }

    #[test]
    fn find_syntax_combined() {
        let result = find_env_vars("something $BITCOIN_IP something ${LND_IP} $ANOTHER_THING");
        let expected = vec!["BITCOIN_IP", "LND_IP", "ANOTHER_THING"];

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

#[cfg(test)]
mod test_flatten {
    use crate::composegenerator::types::Permissions;
    use crate::utils::flatten;

    #[test]
    fn handle_empty_properly() {
        let result = flatten(Vec::<Permissions>::new());
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn handle_simple_properly() {
        let result = flatten(vec![
            Permissions::OneDependency("a".to_string()),
            Permissions::OneDependency("b".to_string()),
        ]);
        assert_eq!(result, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn handle_alternating_properly() {
        let result = flatten(vec![
            Permissions::OneDependency("a".to_string()),
            Permissions::AlternativeDependency(vec!["b".to_string(), "c".to_string()]),
        ]);
        assert_eq!(
            result,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }
}
