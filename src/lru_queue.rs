// TODO: Maybe implement a linked list with WeakRef or hacks (for performance?)
pub struct LruQueue<T> where T: Eq + PartialEq + Clone{
    _queue: Vec<T>,
}

impl<T> LruQueue<T> where T: Eq + PartialEq + Clone{
    pub fn new() -> Self {
        Self {
            _queue: Vec::new(),
        }
    }

    pub fn add_lru(self: &mut LruQueue<T>, new_element: &T){
        // Check if element is already in vec, then flip to the end
        if let Some(position) = self._queue.iter().position(|elem| elem == new_element){
            let elem = self._queue.remove(position);
            self._queue.push(elem);
        } else {
            // Otherwise, add to queue
            self._queue.push(new_element.clone());
        }
    }

    pub fn evict_lru(self: &mut LruQueue<T>) -> Option<T>{
        if self._queue.len() > 0 {
            let result = self._queue.remove(0);
            return Some(result);
        }

        None
    }

    // Remove from lru by element value
    pub fn remove_lru(self: &mut LruQueue<T>, remove_element: &T){
        if let Some(position) = self._queue.iter().position(|elem| elem == remove_element){
            self._queue.remove(position);
        }
    }

}