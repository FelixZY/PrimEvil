// use arraydeque::behavior::Behavior;
// use arraydeque::ArrayDeque;
// use std::cell::RefCell;
// use std::iter::once;
// use std::rc::Rc;
// 
// pub trait OffloadTarget {
//     fn accept_offload(&mut self, offload: &mut dyn DoubleEndedIterator<Item = (u64, u64)>);
// }
// 
// trait BinarySearchable {
//     fn binary_search_by_key(&self, key: u64, start: usize, end: usize) -> Result<usize, usize>;
// }
// 
// impl<const CAP: usize, B: Behavior> BinarySearchable for ArrayDeque<(u64, u64), CAP, B> {
//     fn binary_search_by_key(
//         &self,
//         key: u64,
//         mut low: usize,
//         mut high: usize,
//     ) -> Result<usize, usize> {
//         assert!(high >= low);
//         assert!(high < self.len());
// 
//         while low <= high {
//             let i = low + (high - low) / 2;
//             let key_at_i = self[i].0;
// 
//             if key_at_i == key {
//                 return Ok(i);
//             } else if key_at_i < key {
//                 low = i + 1;
//             } else {
//                 high = match i {
//                     0 => return Err(0),
//                     i => i - 1,
//                 }
//             }
//         }
//         Err(low)
//     }
// }
// 
// pub struct Segment {
//     deque: ArrayDeque<(u64, u64), 5>,
//     min: u64,
//     max: u64,
//     fill: f32,
//     offload_count: usize,
//     offload_target: Rc<RefCell<dyn OffloadTarget>>,
// }
// 
// impl Segment {
//     pub fn new(size: usize, offload_target: Rc<RefCell<dyn OffloadTarget>>) -> Self {
//         assert!(size > 0);
//         Self {
//             deque: ArrayDeque::new(),
//             min: 0,
//             max: 0,
//             fill: 0f32,
//             offload_count: (size / 2).max(1),
//             offload_target,
//         }
//     }
// 
//     pub fn len(&self) -> usize {
//         self.deque.len()
//     }
// 
//     pub fn contains_priority(&self, priority: u64) -> bool {
//         self.deque
//             .binary_search_by_key(priority, 0, self.deque.len() - 1)
//             .is_ok()
//     }
// 
//     pub fn add(&mut self, value: (u64, u64)) {
//         self.offload_if_necessary();
// 
//         match self
//             .deque
//             .binary_search_by_key(value.0, 0, self.deque.len() - 1)
//         {
//             Ok(_) => self
//                 .offload_target
//                 .borrow_mut()
//                 .accept_offload(&mut once(value)),
//             Err(index) => self
//                 .deque
//                 .insert(index, value)
//                 .expect("Insert should succeed"),
//         }
// 
//         self.update_metadata()
//     }
// 
//     fn offload_if_necessary(&mut self) {
//         if self.deque.len() == self.deque.capacity() {
//             self.offload_target.borrow_mut().accept_offload(
//                 &mut self
//                     .deque
//                     .drain((self.deque.len() - self.offload_count)..)
//                     .into_iter(),
//             )
//         }
//     }
// 
//     fn update_metadata(&mut self) {
//         self.min = self.deque.front().map_or(0, |&(priority, _)| priority);
//         self.min = self.deque.back().map_or(0, |&(priority, _)| priority);
//         self.fill = self.deque.len() as f32 / self.deque.capacity() as f32;
//     }
// }
// 
// impl OffloadTarget for Segment {
//     fn accept_offload(&mut self, offload: &mut dyn DoubleEndedIterator<Item = (u64, u64)>) {
//         offload.rev().for_each(|item| {
//             self.offload_if_necessary();
//             self.deque.push_front(item);
//         });
//         self.update_metadata();
//     }
// }
// 
// pub struct PrimerPrioQueue {
//     pub segments: [Rc<RefCell<Segment>>; 5],
// }
// 
// impl PrimerPrioQueue {
//     pub fn new(offload_target: impl OffloadTarget + 'static) -> Self {
//         let offload_target_ref = Rc::new(RefCell::new(offload_target));
//         let mut segments = (2..7).rev().fold(
//             Vec::<Rc<RefCell<Segment>>>::with_capacity(5),
//             |mut segments, x| {
//                 let offload_target: Rc<RefCell<dyn OffloadTarget>> = match segments.last() {
//                     None => offload_target_ref.clone(),
//                     Some(segment) => segment.clone(),
//                 };
//                 segments.push(Rc::new(RefCell::new(Segment::new(
//                     2usize.pow(2 + x),
//                     offload_target,
//                 ))));
//                 segments
//             },
//         );
//         segments.reverse();
//         Self { segments }
//     }
// 
//     pub fn front(&self) -> Option<&(u64, u64)> {
//         self.segments
//             .iter()
//             .find(|x| x.borrow().len() > 0)
//             .map(|x| x.borrow().deque.front())
//             .flatten()
//     }
// 
//     pub fn insert<F>(&mut self, mut value: (u64, u64), uptick: F)
//     where
//         F: Fn(&(u64, u64)) -> (u64, u64),
//     {
//         let mut segment_index: usize = self.get_segment_index(&mut value, &uptick);
//         while self.segments[segment_index]
//             .borrow()
//             .contains_priority(value.0)
//         {
//             value = uptick(&mut value);
//             segment_index = self.get_segment_index(&mut value, &uptick);
//         }
//         self.segments[segment_index].borrow_mut().add(value)
//     }
// 
//     fn get_segment_index<F>(&mut self, value: &mut (u64, u64), uptick: &F) -> usize
//     where
//         F: Fn(&(u64, u64)) -> (u64, u64),
//     {
//         let mut segment_index_result: Result<usize, usize> = Ok(0);
//         while segment_index_result.is_ok() {
//             segment_index_result = self
//                 .segments
//                 .binary_search_by_key(&value.0, |segment| segment.borrow().max);
//             if segment_index_result.is_ok() {
//                 (value.0, value.1) = uptick(value);
//             }
//         }
//         let mut segment_index = segment_index_result
//             .unwrap_err()
//             .min(self.segments.len() - 1);
// 
//         if segment_index + 1 < self.segments.len()
//             && self.segments[segment_index].borrow().fill > 0.5
//             && self.segments[segment_index].borrow().fill
//                 < self.segments[segment_index + 1].borrow().fill
//         {
//             segment_index += 1;
//         }
// 
//         segment_index
//     }
// }
