use std::cell::Cell;
use std::rc::Rc;

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
        let handle = Handle(Rc::new(Cell::new(idx)));
        self.items.push(Wrapper {
            item,
            handle: handle.clone(),
        });
        self.percolate_up(idx);
        handle
    }

    pub fn pop_max(&mut self) -> Option<T> {
        unimplemented!()
    }

    pub fn peek_max(&mut self) -> Option<&T> {
        self.items.first().map(|wrapper| &wrapper.item)
    }

    pub fn increment<F: FnOnce(&mut T)>(&mut self, handle: &Handle, f: F) {
        println!("incrementing item @ {:?}", handle);
        f(&mut self.items[handle.0.get()].item);
        self.percolate_up(handle.0.get())
    }

    pub fn decrement<F: FnOnce(&mut T)>(&mut self, handle: &Handle, f: F) {
        println!("decrementing item @ {:?}", handle);
        f(&mut self.items[handle.0.get()].item);
        self.percolate_down(handle.0.get())
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
            self.items[idx].handle.0.set(lowest);
            self.items[lowest].handle.0.set(idx);
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
            self.items[idx].handle.0.set(highest);
            self.items[highest].handle.0.set(idx);
            self.items.swap(idx, highest);
            idx = highest;
        }
    }
}

#[derive(Clone, Debug)]
pub struct Handle(Rc<Cell<usize>>);

struct Wrapper<T> {
    item: T,
    handle: Handle,
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
}
