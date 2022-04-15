#[gobject::class(final, implements(gtk::gio::ListModel))]
mod basic_group_list {
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use indexmap::map::Entry;
    use indexmap::IndexMap;
    use std::cell::RefCell;
    use tdlib::enums::Update;

    use crate::session::BasicGroup;

    #[derive(Default)]
    pub(crate) struct BasicGroupList {
        list: RefCell<IndexMap<i64, BasicGroup>>,
    }

    impl ListModelImpl for BasicGroupList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            BasicGroup::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .get_index(position as usize)
                .map(|(_, o)| o.clone().upcast::<glib::Object>())
        }
    }

    impl super::BasicGroupList {
        pub(crate) fn new() -> Self {
            glib::Object::new(&[]).unwrap()
        }

        /// Return the `BasicGroup` of the specified `id`. Panics if the basic group is not present.
        /// Note that TDLib guarantees that objects are always returned before of their ids,
        /// so if you use an `id` returned by TDLib, it should be expected that the relative
        /// `BasicGroup` exists in the list.
        pub(crate) fn get(&self, id: i64) -> BasicGroup {
            self.imp().list.borrow().get(&id).unwrap().clone()
        }

        pub(crate) fn handle_update(&self, update: Update) {
            if let Update::BasicGroup(data) = update {
                let mut list = self.imp().list.borrow_mut();

                match list.entry(data.basic_group.id) {
                    Entry::Occupied(entry) => entry.get().handle_update(Update::BasicGroup(data)),
                    Entry::Vacant(entry) => {
                        let basic_group = data.basic_group.into();
                        entry.insert(basic_group);

                        let index = list.len() - 1;
                        drop(list);

                        self.items_changed(index as u32, 0, 1);
                    }
                }
            }
        }
    }
}
