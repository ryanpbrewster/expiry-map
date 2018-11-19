use std::cell::Cell;
use std::rc::{Rc, Weak};

#[derive(Default)]
pub struct MutHeap<T>
where
    T: Ord,
{
    items: Vec<Wrapper<T>>,
}

impl<T> MutHeap<T>
where
    T: Ord,
{
    pub fn insert(&mut self, item: T) -> Handle {
        let idx = self.items.len();
        let handle = Rc::new(Cell::new(idx));
        let ret = Handle(Rc::downgrade(&handle));
        self.items.push(Wrapper { item, handle });
        self.percolate_up(idx);
        ret
    }

    pub fn pop_max(&mut self) -> Option<T> {
        self.items.pop().map(|mut tmp| {
            if !self.items.is_empty() {
                ::std::mem::swap(&mut self.items[0], &mut tmp);
                self.percolate_down(0);
            }
            tmp.item
        })
    }

    pub fn peek_max(&mut self) -> Option<&T> {
        self.items.first().map(|wrapper| &wrapper.item)
    }

    pub fn increment<F: FnOnce(&mut T)>(&mut self, handle: &Handle, f: F) {
        println!("incrementing item @ {:?}", handle);
        let idx = handle
            .0
            .upgrade()
            .expect("handle not present in heap")
            .get();
        f(&mut self.items[idx].item);
        self.percolate_up(idx);
    }

    pub fn decrement<F: FnOnce(&mut T)>(&mut self, handle: &Handle, f: F) {
        println!("decrementing item @ {:?}", handle);
        let idx = handle
            .0
            .upgrade()
            .expect("handle not present in heap")
            .get();
        f(&mut self.items[idx].item);
        self.percolate_down(idx);
    }

    fn percolate_up(&mut self, mut idx: usize) {
        while idx > 0 {
            let mut lowest = idx;
            if self.items[idx / 2].item < self.items[idx].item {
                lowest = idx / 2;
            }
            if lowest == idx {
                break;
            }
            self.items[idx].handle.set(lowest);
            self.items[lowest].handle.set(idx);
            self.items.swap(idx, lowest);
            idx = lowest;
        }
    }
    fn percolate_down(&mut self, mut idx: usize) {
        while 2 * idx < self.items.len() {
            let mut highest = idx;
            if self.items[2 * idx].item > self.items[highest].item {
                highest = 2 * idx;
            }
            if 2 * idx + 1 < self.items.len()
                && self.items[2 * idx + 1].item > self.items[highest].item
            {
                highest = 2 * idx + 1;
            }
            if highest == idx {
                break;
            }
            self.items[idx].handle.set(highest);
            self.items[highest].handle.set(idx);
            self.items.swap(idx, highest);
            idx = highest;
        }
    }
}

#[derive(Clone, Debug)]
pub struct Handle(Weak<Cell<usize>>);

struct Wrapper<T> {
    item: T,
    handle: Rc<Cell<usize>>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke_test() {
        let mut heap = MutHeap::default();

        let a = heap.insert(10);
        heap.insert(20);

        assert_eq!(heap.peek_max(), Some(&20));

        heap.increment(&a, |x| {
            *x += 100;
        });
        assert_eq!(heap.peek_max(), Some(&110));

        heap.decrement(&a, |x| {
            *x -= 100;
        });
        assert_eq!(heap.peek_max(), Some(&20));
    }

    #[test]
    #[should_panic]
    fn panic_on_expired_handle() {
        let mut heap = MutHeap::default();

        let a = heap.insert(10);
        assert_eq!(heap.pop_max(), Some(10));
        heap.insert(20);

        heap.increment(&a, |_| ());
    }
}
