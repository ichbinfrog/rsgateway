#[derive(Debug)]
pub struct Entry<T> {
    value: Option<T>,

    prev: usize,
    next: usize,
}

#[derive(Debug)]
pub struct LinkedList<T> {
    entries: Vec<Entry<T>>,
    length: usize,
    capacity: usize,
}

impl<T> LinkedList<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        let mut entries = Vec::with_capacity(capacity);
        entries.push(Entry {
            value: None,
            next: 0,
            prev: 0,
        });
        entries.push(Entry {
            value: None,
            next: 1,
            prev: 1,
        });
        Self {
            entries,
            length: 0,
            capacity,
        }
    }

    pub fn unlink(&mut self, index: usize) {
        let prev = self.entries[index].prev;
        let next = self.entries[index].next;
        self.entries[prev].next = next;
        self.entries[next].prev = prev;
    }

    pub fn link_after(&mut self, index: usize, prev: usize) {
        let next = self.entries[prev].next;
        self.entries[next].prev = prev;
        self.entries[next].next = next;
        self.entries[prev].next = index;
        self.entries[next].prev = index;
    }

    pub fn push_front(&mut self, value: T) -> usize {
        if self.entries[0].next == 0 {
            self.entries.push(Entry {
                value: None,
                next: 0,
                prev: 0,
            });
            self.entries[0].next = self.entries.len() - 1;
        }
        let index = self.entries[0].next;
        self.entries[index].value = Some(value);
        self.unlink(index);
        self.link_after(index, 1);
        index
    }

    pub fn back(&self) -> usize {
        self.entries[1].prev
    }

    pub fn get(&self, index: usize) -> &T {
        self.entries[index].value.as_ref().expect("invalid index")
    }

    pub fn iter(&self) -> LinkedListIterator<T> {
        LinkedListIterator::<T> {
            list: self,
            index: 1,
        }
    }
}

#[derive(Debug)]
pub struct LinkedListIterator<'a, T> {
    list: &'a LinkedList<T>,
    index: usize,
}

impl<'a, T> Iterator for LinkedListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.list.entries[self.index].next;
        if next == 0 {
            None
        } else {
            let value = self.list.entries[next].value.as_ref();
            self.index = next;
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LinkedList;

    #[test]
    fn test_lru() {
        let mut lru = LinkedList::<&str>::with_capacity(10);
        println!("{:?}", lru.entries);

        lru.push_front("Holla");
        println!("{:?}", lru.entries);

        lru.push_front("Quetal");
        println!("{:?}", lru.entries);

        lru.push_front("Comom");
        println!("{:?}", lru.entries);

        lru.push_front("Holla");
        println!("{:?}", lru.entries);

        println!("{:?}", lru.get(lru.back()));
    }
}
