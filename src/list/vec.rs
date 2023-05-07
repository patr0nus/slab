use super::{AsSliceListStorage, ClearableListStorage, ListStorage, MutRefListStorage};

#[cfg(not(slab_no_gat))]
use super::{ItemMut, List};

#[derive(Debug)]
pub enum VecStorage {}

impl ListStorage for VecStorage {
    #[cfg(not(slab_no_gat))]
    type List<T> = Vec<T>;
}

impl ClearableListStorage for VecStorage {
    #[cfg(not(slab_no_gat))]
    fn clear<T>(list: &mut Self::List<T>) {
        list.clear()
    }
}
impl MutRefListStorage for VecStorage {
    #[cfg(not(slab_no_gat))]
    fn into_mut_ref<T>(
        item_mut: <<Self as ListStorage>::List<T> as List>::ItemMut<'_>,
    ) -> &'_ mut T {
        item_mut
    }
}

impl AsSliceListStorage for VecStorage {
    #[cfg(not(slab_no_gat))]
    fn as_slice<T>(list: &Self::List<T>) -> &[T] {
        list.as_slice()
    }
    #[cfg(not(slab_no_gat))]
    fn as_mut_slice<T>(list: &mut Self::List<T>) -> &mut [T] {
        list.as_mut_slice()
    }
}

#[cfg(not(slab_no_gat))]
impl<T> List for Vec<T> {
    type Item = T;
    type ItemMut<'a> = &'a mut T where Self: 'a;

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn push(&mut self, item: Self::Item) {
        Vec::push(self, item);
    }

    fn get(&self, key: usize) -> Option<&Self::Item> {
        self.as_slice().get(key)
    }
    fn get_mut(&mut self, key: usize) -> Option<Self::ItemMut<'_>> {
        self.as_mut_slice().get_mut(key)
    }
}
