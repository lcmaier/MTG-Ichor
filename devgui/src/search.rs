//! A box that narrows a long list of names as the person types: the board
//! editor's card search, and the one a prompt to choose a card name can take
//! (CR 201.4, `backlog.md` §2.4). The list is the engine's; narrowing it is
//! one client's way of picking from it.

/// A list of names, and those holding what the person typed, case aside.
#[derive(Clone, Debug)]
pub struct NameSearch {
    names: Vec<String>,
    /// Each name in lower case, matched against.
    folded: Vec<String>,
    query: String,
    /// The places in `names` of the names holding the query, in list order.
    matches: Vec<usize>,
}

impl NameSearch {
    pub fn new(names: Vec<String>) -> NameSearch {
        let folded = names.iter().map(|name| name.to_lowercase()).collect();
        let matches = (0..names.len()).collect();
        NameSearch { names, folded, query: String::new(), matches }
    }

    /// Narrow the list to the names holding `query`, ignoring case and the
    /// spaces around it; an empty query keeps every name.
    pub fn set_query(&mut self, query: String) {
        let wanted = query.trim().to_lowercase();
        self.matches = (0..self.names.len()).filter(|&i| self.folded[i].contains(&wanted)).collect();
        self.query = query;
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn matches(&self) -> &[usize] {
        &self.matches
    }

    /// The name at place `i` in the list.
    pub fn name(&self, i: usize) -> Option<&str> {
        self.names.get(i).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_query_keeps_the_names_holding_it_whatever_their_case() {
        let names = ["Grizzly Bears", "Bear Cub", "Forest", "Isamaru, Hound of Konda"];
        let mut search = NameSearch::new(names.iter().map(|n| n.to_string()).collect());
        assert_eq!(search.matches(), [0, 1, 2, 3], "every name before anything is typed");
        search.set_query(" BEAR ".to_string());
        assert_eq!(search.matches(), [0, 1], "in the list's order");
        search.set_query("hound of".to_string());
        assert_eq!(search.matches().iter().map(|&i| search.name(i).unwrap()).collect::<Vec<_>>(), ["Isamaru, Hound of Konda"]);
        search.set_query("Vampire".to_string());
        assert!(search.matches().is_empty());
        assert_eq!(search.query(), "Vampire");
    }
}
