struct LinkedList<'e, T> {
    head: LinkedListElement<'e, T>,
    tail: &'e mut LinkedListElement<'e, T>,
    len: usize,
}

impl<'e, T> LinkedList<'e, T> {
    pub fn new(initial: T) -> LinkedList<'e, T> {
        let mut element = LinkedListElement {
            next: None,
            prev: None,
            value: initial,
        };
        LinkedList {
            head: element,
            tail: &mut element,
            len: 1,
        }
    }

    pub fn add(&mut self, value: T) {
        let mut element = LinkedListElement {
            value,
            next: None,
            prev: Some(self.tail),
        };
        self.tail.next = Some(&mut element);
        self.tail = &mut element;
    }

    pub fn iter(&self) -> LinkedListIterator<'e, T> {
        LinkedListIterator {
            linked_list: &self,
            current: &&self.head,
        }
    }
}

struct LinkedListElement<'a, T> {
    pub value: T,
    pub next: Option<&'a mut LinkedListElement<'a, T>>,
    pub prev: Option<&'a mut LinkedListElement<'a, T>>,
}

struct LinkedListIterator<'c, T> {
    linked_list: &'c LinkedList<'c, T>,
    current: &'c LinkedListElement<'c, T>,
}

impl<'e, T> Iterator for LinkedListIterator<'e, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let el = self.current.next;
        match el {
            None => None,
            Some(t) => {
                self.current = t;
                Some(t.value)
            }
        }
    }
}
