// Maybe implement a linked list with WeakRef or hacks (for performance?)
// Lecturer allows Vec #765
pub struct LruQueue<T>
where
    T: Eq + PartialEq + Clone,
{
    queue: Vec<T>,
}

impl<T> LruQueue<T>
where
    T: Eq + PartialEq + Clone,
{
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn add_lru(self: &mut LruQueue<T>, new_element: &T) {
        // Check if element is already in vec, then flip to the end
        if let Some(position) = self.queue.iter().position(|elem| elem == new_element) {
            let elem = self.queue.remove(position);
            self.queue.push(elem);
        } else {
            // Otherwise, add to queue
            self.queue.push(new_element.clone());
        }
    }

    pub fn evict_lru(self: &mut LruQueue<T>) -> Option<T> {
        if self.queue.len() > 0 {
            let result = self.queue.remove(0);
            return Some(result);
        }

        None
    }

    pub fn evict_lru_by_value(self: &mut LruQueue<T>, value: &T) -> Option<T> {
        let position = self.queue.iter().position(|x| x == value)?;
        let result = self.queue.remove(position);
        Some(result)
    }
}
