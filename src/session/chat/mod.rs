mod action;
mod action_list;
mod history;
mod item;
mod message;
mod message_forward_info;
mod sponsored_message;

pub(crate) use self::action::ChatAction;
pub(crate) use self::action_list::ChatActionList;
use self::history::History;
pub(crate) use self::item::{Item, ItemType};
pub(crate) use self::message::{Message, MessageSender};
pub(crate) use self::message_forward_info::{MessageForwardInfo, MessageForwardOrigin};
pub(crate) use self::sponsored_message::SponsoredMessage;

use gtk::glib;
use tdlib::enums::{ChatList, ChatType as TdChatType, MessageContent};
use tdlib::types::{ChatNotificationSettings, DraftMessage};

use crate::session::{BasicGroup, SecretChat, Supergroup, User};
use crate::Session;

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "ChatType")]
pub(crate) enum ChatType {
    Private(User),
    BasicGroup(BasicGroup),
    Supergroup(Supergroup),
    Secret(SecretChat),
}

impl ChatType {
    pub(crate) fn from_td_object(_type: &TdChatType, session: &Session) -> Self {
        match _type {
            TdChatType::Private(data) => {
                let user = session.user_list().get(data.user_id);
                Self::Private(user)
            }
            TdChatType::BasicGroup(data) => {
                let basic_group = session.basic_group_list().get(data.basic_group_id);
                Self::BasicGroup(basic_group)
            }
            TdChatType::Supergroup(data) => {
                let supergroup = session.supergroup_list().get(data.supergroup_id);
                Self::Supergroup(supergroup)
            }
            TdChatType::Secret(data) => {
                let secret_chat = session.secret_chat_list().get(data.secret_chat_id);
                Self::Secret(secret_chat)
            }
        }
    }

    pub(crate) fn user(&self) -> Option<&User> {
        Some(match self {
            ChatType::Private(user) => user,
            ChatType::Secret(secret_chat) => secret_chat.user(),
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedDraftMessage", nullable)]
pub(crate) struct BoxedDraftMessage(pub(crate) DraftMessage);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedChatNotificationSettings")]
pub(crate) struct BoxedChatNotificationSettings(pub(crate) ChatNotificationSettings);

#[derive(Clone, Debug, PartialEq, glib::Boxed)]
#[boxed_type(name = "BoxedMessageContent")]
pub(crate) struct BoxedMessageContent(pub(crate) MessageContent);

#[gobject::class(final)]
mod chat {
    use super::*;
    use gobject::{ConstructCell, WeakCell};
    use gtk::subclass::prelude::*;
    use once_cell::unsync::OnceCell;
    use std::cell::{Cell, RefCell};
    use tdlib::enums::Update;
    use tdlib::types::Chat as TdChat;

    use crate::session::Avatar;
    use crate::Session;

    #[derive(Default)]
    pub(crate) struct Chat {
        #[property(get)]
        id: Cell<i64>,
        #[property(get, boxed)]
        type_: OnceCell<ChatType>,
        #[property(get)]
        title: RefCell<String>,
        #[property(get, boxed)]
        avatar: RefCell<Option<Avatar>>,
        #[property(get, object)]
        last_message: RefCell<Option<Message>>,
        #[property(get)]
        order: Cell<i64>,
        #[property(get)]
        is_pinned: Cell<bool>,
        #[property(get)]
        unread_count: Cell<i32>,
        #[property(get)]
        unread_mention_count: Cell<i32>,
        #[property(get, boxed)]
        notification_settings: ConstructCell<BoxedChatNotificationSettings>,
        #[property(get, boxed)]
        draft_message: RefCell<Option<BoxedDraftMessage>>,
        #[property(get = "_", object)]
        history: OnceCell<History>,
        #[property(get = "_", object)]
        actions: OnceCell<ChatActionList>,
        #[property(get, object)]
        session: WeakCell<Session>,
    }

    impl Chat {
        #[public]
        fn history(&self) -> &History {
            self.history.get_or_init(|| History::new(&self.instance()))
        }

        #[public]
        fn actions(&self) -> &ChatActionList {
            self.actions
                .get_or_init(|| ChatActionList::from(&self.instance()))
        }
    }

    impl super::Chat {
        pub(crate) fn from_td_object(td_object: TdChat, session: &Session) -> Self {
            let obj: Self = glib::Object::new(&[]).unwrap();
            let imp = obj.imp();

            imp.id.set(td_object.id);
            imp.type_
                .set(ChatType::from_td_object(&td_object.r#type, &session))
                .unwrap();
            imp.title.replace(td_object.title);
            imp.avatar.replace(td_object.photo.map(Avatar::from));
            // TODO: Last Message
            // TODO: Order
            // TODO: Is Pinned
            imp.unread_count.set(td_object.unread_count);
            imp.unread_mention_count.set(td_object.unread_mention_count);
            imp.notification_settings
                .replace(Some(BoxedChatNotificationSettings(
                    td_object.notification_settings,
                )));
            imp.draft_message
                .replace(td_object.draft_message.map(BoxedDraftMessage));
            imp.session.set(Some(session));

            obj
        }

        pub(crate) fn handle_update(&self, update: Update) {
            match update {
                Update::NewMessage(_)
                | Update::MessageSendSucceeded(_)
                | Update::MessageContent(_)
                | Update::DeleteMessages(_) => {
                    self.history().handle_update(update);
                }
                Update::ChatTitle(update) => {
                    self.set_title(update.title);
                }
                Update::ChatPhoto(update) => {
                    self.set_avatar(update.photo.map(Into::into));
                }
                Update::ChatLastMessage(update) => {
                    match update.last_message {
                        Some(last_message) => {
                            let message = match self.history().message_by_id(last_message.id) {
                                Some(message) => message,
                                None => {
                                    let last_message_id = last_message.id;

                                    self.history().append(last_message);
                                    self.history().message_by_id(last_message_id).unwrap()
                                }
                            };

                            self.set_last_message(Some(message));
                        }
                        None => self.set_last_message(None),
                    }

                    for position in update.positions {
                        if let ChatList::Main = position.list {
                            self.set_order(position.order);
                            break;
                        }
                    }
                }
                Update::ChatNotificationSettings(update) => {
                    self.set_notification_settings(BoxedChatNotificationSettings(
                        update.notification_settings,
                    ));
                }
                Update::ChatPosition(update) => {
                    if let ChatList::Main = update.position.list {
                        self.set_order(update.position.order);
                        self.set_is_pinned(update.position.is_pinned);
                    }
                }
                Update::ChatUnreadMentionCount(update) => {
                    self.set_unread_mention_count(update.unread_mention_count);
                }
                Update::MessageMentionRead(update) => {
                    self.set_unread_mention_count(update.unread_mention_count);
                }
                Update::ChatReadInbox(update) => {
                    self.set_unread_count(update.unread_count);
                }
                Update::ChatDraftMessage(update) => {
                    self.set_draft_message(update.draft_message.map(BoxedDraftMessage));
                }
                Update::ChatAction(update) => {
                    self.actions().handle_update(update);
                    // TODO: Remove this at some point. Widgets should use the `items-changed` signal
                    // for updating their state in the future.
                    self.notify_actions();
                }
                _ => {}
            }
        }

        pub(crate) fn is_own_chat(&self) -> bool {
            self.type_().user() == Some(&self.session().me())
        }

        fn set_title(&self, title: String) {
            let imp = self.imp();
            let old = imp.title.replace(title);
            if old != *imp.title.borrow() {
                self.notify_title();
            }
        }

        fn set_avatar(&self, avatar: Option<Avatar>) {
            let imp = self.imp();
            let old = imp.avatar.replace(avatar);
            if old != *imp.avatar.borrow() {
                self.notify_avatar();
            }
        }

        fn set_last_message(&self, last_message: Option<Message>) {
            let imp = self.imp();
            let old = imp.last_message.replace(last_message);
            if old != *imp.last_message.borrow() {
                self.notify_last_message();
            }
        }

        fn set_order(&self, order: i64) {
            let imp = self.imp();
            let old = imp.order.replace(order);
            if old != imp.order.get() {
                self.notify_order();
            }
        }

        fn set_is_pinned(&self, is_pinned: bool) {
            let imp = self.imp();
            let old = imp.is_pinned.replace(is_pinned);
            if old != imp.is_pinned.get() {
                self.notify_is_pinned();
            }
        }

        fn set_unread_count(&self, unread_count: i32) {
            let imp = self.imp();
            let old = imp.unread_count.replace(unread_count);
            if old != imp.unread_count.get() {
                self.notify_unread_count();
            }
        }

        fn set_unread_mention_count(&self, unread_mention_count: i32) {
            let imp = self.imp();
            let old = imp.unread_mention_count.replace(unread_mention_count);
            if old != imp.unread_mention_count.get() {
                self.notify_unread_mention_count();
            }
        }

        fn set_notification_settings(&self, notification_settings: BoxedChatNotificationSettings) {
            let imp = self.imp();
            let old = imp
                .notification_settings
                .replace(Some(notification_settings));
            if old != *imp.notification_settings.borrow() {
                self.notify_notification_settings();
            }
        }

        fn set_draft_message(&self, draft_message: Option<BoxedDraftMessage>) {
            let imp = self.imp();
            let old = imp.draft_message.replace(draft_message);
            if old != *imp.draft_message.borrow() {
                self.notify_draft_message();
            }
        }
    }
}
