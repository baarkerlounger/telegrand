use gtk::glib;
use tdlib::enums::{UserStatus, UserType};

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedUserType")]
pub(crate) struct BoxedUserType(pub(crate) UserType);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedUserStatus")]
pub(crate) struct BoxedUserStatus(pub(crate) UserStatus);

#[gobject::class(final)]
mod user {
    use super::*;
    use gobject::{ConstructCell, WeakCell};
    use gtk::subclass::prelude::*;
    use std::cell::{Cell, RefCell};
    use tdlib::enums::Update;
    use tdlib::types::User as TdUser;

    use crate::session::Avatar;
    use crate::Session;

    #[derive(Default)]
    pub(crate) struct User {
        #[property(get)]
        id: Cell<i64>,
        #[property(get)]
        first_name: RefCell<String>,
        #[property(get)]
        last_name: RefCell<String>,
        #[property(get)]
        username: RefCell<String>,
        #[property(get)]
        phone_number: RefCell<String>,
        #[property(get, boxed)]
        status: ConstructCell<BoxedUserStatus>,
        #[property(get, boxed)]
        avatar: RefCell<Option<Avatar>>,
        #[property(get, boxed)]
        type_: ConstructCell<BoxedUserType>,
        #[property(get, object)]
        session: WeakCell<Session>,
    }

    impl super::User {
        pub(crate) fn from_td_object(td_object: TdUser, session: &Session) -> Self {
            let obj: Self = glib::Object::new(&[]).unwrap();
            let imp = obj.imp();

            imp.id.set(td_object.id);
            imp.first_name.replace(td_object.first_name);
            imp.last_name.replace(td_object.last_name);
            imp.username.replace(td_object.username);
            imp.phone_number.replace(td_object.phone_number);
            imp.status.replace(Some(BoxedUserStatus(td_object.status)));
            imp.avatar
                .replace(td_object.profile_photo.map(Avatar::from));
            imp.type_.replace(Some(BoxedUserType(td_object.r#type)));
            imp.session.set(Some(session));

            obj
        }

        pub(crate) fn handle_update(&self, update: Update) {
            match update {
                Update::User(data) => {
                    self.set_first_name(data.user.first_name);
                    self.set_last_name(data.user.last_name);
                    self.set_username(data.user.username);
                    self.set_phone_number(data.user.phone_number);
                    self.set_status(BoxedUserStatus(data.user.status));
                    self.set_avatar(data.user.profile_photo.map(Avatar::from));
                    self.set_type(BoxedUserType(data.user.r#type));
                }
                Update::UserStatus(data) => self.set_status(BoxedUserStatus(data.status)),
                _ => {}
            }
        }

        fn set_first_name(&self, first_name: String) {
            let imp = self.imp();
            let old = imp.first_name.replace(first_name);
            if old != *imp.first_name.borrow() {
                self.notify_first_name();
            }
        }

        fn set_last_name(&self, last_name: String) {
            let imp = self.imp();
            let old = imp.last_name.replace(last_name);
            if old != *imp.last_name.borrow() {
                self.notify_last_name();
            }
        }

        fn set_username(&self, username: String) {
            let imp = self.imp();
            let old = imp.username.replace(username);
            if old != *imp.username.borrow() {
                self.notify_username();
            }
        }

        fn set_phone_number(&self, phone_number: String) {
            let imp = self.imp();
            let old = imp.phone_number.replace(phone_number);
            if old != *imp.phone_number.borrow() {
                self.notify_phone_number();
            }
        }

        fn set_status(&self, status: BoxedUserStatus) {
            let imp = self.imp();
            let old = imp.status.replace(Some(status));
            if old != *imp.status.borrow() {
                self.notify_status();
            }
        }

        fn set_avatar(&self, avatar: Option<Avatar>) {
            let imp = self.imp();
            let old = imp.avatar.replace(avatar);
            if old != *imp.avatar.borrow() {
                self.notify_avatar();
            }
        }

        fn set_type(&self, type_: BoxedUserType) {
            let imp = self.imp();
            let old = imp.type_.replace(Some(type_));
            if old != *imp.type_.borrow() {
                self.notify_type();
            }
        }
    }
}
