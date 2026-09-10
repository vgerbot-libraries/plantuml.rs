/// Immutable list wrapper used by `Capture` for storing capture entries.
///
/// In the Java original this is a custom linked-list with block caching for
/// efficient immutable `add` / `addAll` operations.  In Rust we simplify to a
/// `Vec`-backed wrapper that clones on every mutation, preserving the
/// immutable-semantics of the Java API.
///
/// Ported from: `com/plantuml/ubrex/SafeList.java`

use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, PartialEq, Eq)]
pub struct SafeList<E: Clone> {
    items: Vec<E>,
}

impl<E: Clone + Debug> Debug for SafeList<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(&self.items).finish()
    }
}

impl<E: Clone> SafeList<E> {
    /// Creates an empty list.
    pub fn create_empty() -> Self {
        SafeList { items: Vec::new() }
    }

    /// Returns a new list with `item` appended.
    pub fn add(&self, item: E) -> Self {
        let mut new_items = self.items.clone();
        new_items.push(item);
        SafeList { items: new_items }
    }

    /// Returns the number of elements.
    pub fn size(&self) -> usize {
        self.items.len()
    }

    /// Returns a new list with all elements of `other` appended.
    pub fn add_all(&self, other: &SafeList<E>) -> Self {
        let mut new_items = self.items.clone();
        new_items.extend(other.items.iter().cloned());
        SafeList { items: new_items }
    }

    /// Returns a new list with `f` applied to every element.
    pub fn mapped<F: Fn(&E) -> E>(&self, f: F) -> Self {
        SafeList {
            items: self.items.iter().map(f).collect(),
        }
    }

    /// Returns an iterator over the elements.
    pub fn iter(&self) -> impl Iterator<Item = &E> {
        self.items.iter()
    }

    /// Returns `true` if the list contains no elements.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<E: Clone + Display> Display for SafeList<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ported from: `com/plantuml/ubrex/SafeListTest.java`
    #[test]
    fn test1() {
        assert_eq!(0, SafeList::<String>::create_empty().size());
    }

    /// Ported from: `SafeListTest.testAdd`
    #[test]
    fn test_add() {
        let list1 = SafeList::<String>::create_empty();
        let list2 = list1.add("Alpha".to_string());
        let list3 = list2.add("Beta".to_string());

        assert_eq!(0, list1.size());
        assert_eq!(1, list2.size());
        assert_eq!(2, list3.size());

        assert_eq!("[]", list1.to_string());
        assert_eq!("[Alpha]", list2.to_string());
        assert_eq!("[Alpha, Beta]", list3.to_string());
    }

    /// Ported from: `SafeListTest.testMerge`
    #[test]
    fn test_merge() {
        let list_abc = SafeList::<String>::create_empty()
            .add("A".to_string())
            .add("B".to_string())
            .add("C".to_string());
        let list_de = SafeList::<String>::create_empty()
            .add("D".to_string())
            .add("E".to_string());

        assert_eq!("[A, B, C]", list_abc.to_string());
        assert_eq!("[D, E]", list_de.to_string());

        let list_abcde = list_abc.add_all(&list_de);
        assert_eq!("[A, B, C]", list_abc.to_string());
        assert_eq!("[D, E]", list_de.to_string());
        assert_eq!("[A, B, C, D, E]", list_abcde.to_string());
    }
}
