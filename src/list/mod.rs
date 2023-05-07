#[cfg(not(slab_no_gat))]
pub trait ItemMut<'a, T> {
    fn set(&mut self, item: T);
    fn get(&self) -> &T;
}

#[cfg(not(slab_no_gat))]
pub trait ListStorage {
    type List<T>: List<Item = T>;
}

#[cfg(slab_no_gat)]
pub trait ListStorage { }

#[cfg(not(slab_no_gat))]
pub trait ClearableListStorage: ListStorage {
    fn clear<T>(list: &mut Self::List<T>);
}

#[cfg(slab_no_gat)]
pub trait ClearableListStorage: ListStorage { }

#[cfg(not(slab_no_gat))]
pub trait AsSliceListStorage: ListStorage {
    fn as_slice<T>(list: &Self::List<T>) -> &[T];
    fn as_mut_slice<T>(list: &mut Self::List<T>) -> &mut [T];
}

#[cfg(slab_no_gat)]
pub trait AsSliceListStorage: ListStorage { }

#[cfg(not(slab_no_gat))]
pub trait MutRefListStorage: ListStorage {
    fn into_mut_ref<T>(
        item_mut: <<Self as ListStorage>::List<T> as List>::ItemMut<'_>,
    ) -> &'_ mut T;
}

#[cfg(slab_no_gat)]
pub trait MutRefListStorage: ListStorage { }

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
pub trait List {
    type Item;
    type ItemMut<'a>: ItemMut<'a, Self::Item>
    where
        Self: 'a;
    fn len(&self) -> usize;
    fn push(&mut self, item: Self::Item);
    fn get(&self, key: usize) -> Option<&Self::Item>;
    fn get_mut(&mut self, key: usize) -> Option<Self::ItemMut<'_>>;
}

#[cfg(not(slab_no_gat))]
impl<'a, T> ItemMut<'a, T> for &'a mut T {
    fn set(&mut self, item: T) {
        **self = item;
    }
    fn get(&self) -> &T {
        &*self
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
