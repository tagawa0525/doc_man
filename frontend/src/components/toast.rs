use leptos::prelude::*;

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u32,
    pub message: String,
    pub kind: ToastKind,
}

#[derive(Debug, Clone)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

impl ToastKind {
    fn class(&self) -> &'static str {
        match self {
            ToastKind::Success => "is-success",
            ToastKind::Error => "is-danger",
            ToastKind::Info => "is-info",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ToastContext {
    toasts: RwSignal<Vec<Toast>>,
    next_id: RwSignal<u32>,
}

impl ToastContext {
    pub fn new() -> Self {
        Self {
            toasts: RwSignal::new(Vec::new()),
            next_id: RwSignal::new(0),
        }
    }

    pub fn success(&self, message: impl Into<String>) {
        self.add(message.into(), ToastKind::Success);
    }

    pub fn error(&self, message: impl Into<String>) {
        self.add(message.into(), ToastKind::Error);
    }

    pub fn info(&self, message: impl Into<String>) {
        self.add(message.into(), ToastKind::Info);
    }

    fn add(&self, message: String, kind: ToastKind) {
        let id = self.next_id.get_untracked();
        self.next_id.set(id + 1);
        self.toasts.update(|t| t.push(Toast { id, message, kind }));

        let toasts = self.toasts;
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(3000).await;
            toasts.update(|t| t.retain(|toast| toast.id != id));
        });
    }
}

#[component]
pub fn ToastContainer() -> impl IntoView {
    let ctx = expect_context::<ToastContext>();

    view! {
        <div class="toast-container">
            {move || {
                ctx.toasts.get().into_iter().map(|toast| {
                    let class = format!("notification toast-item {}", toast.kind.class());
                    let id = toast.id;
                    let toasts = ctx.toasts;
                    view! {
                        <div class=class>
                            <button class="delete" on:click=move |_| {
                                toasts.update(|t| t.retain(|t| t.id != id));
                            }></button>
                            {toast.message}
                        </div>
                    }
                }).collect_view()
            }}
        </div>
    }
}
