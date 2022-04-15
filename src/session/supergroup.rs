#[gobject::class(final)]
mod supergroup {
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use std::cell::Cell;
    use tdlib::enums::Update;
    use tdlib::types::Supergroup as TdSupergroup;

    #[derive(Default)]
    pub(crate) struct Supergroup {
        #[property(get)]
        id: Cell<i64>,
        #[property(get)]
        member_count: Cell<i32>,
        #[property(get)]
        is_channel: Cell<bool>,
    }

    impl super::Supergroup {
        pub(crate) fn handle_update(&self, update: Update) {
            if let Update::Supergroup(data) = update {
                self.set_member_count(data.supergroup.member_count);
            }
        }

        fn set_member_count(&self, member_count: i32) {
            let imp = self.imp();
            let old = imp.member_count.replace(member_count);
            if old != imp.member_count.get() {
                self.notify_member_count();
            }
        }
    }

    impl From<TdSupergroup> for super::Supergroup {
        fn from(td_object: TdSupergroup) -> Self {
            let obj: Self = glib::Object::new(&[]).unwrap();
            let imp = obj.imp();

            imp.id.set(td_object.id);
            imp.member_count.set(td_object.member_count);
            imp.is_channel.set(td_object.is_channel);

            obj
        }
    }
}
