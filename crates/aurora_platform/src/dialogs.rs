use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// A handle to a file dialog running on a background thread.
///
/// Returned by the `*_async` methods on [`FileDialog`]. Poll with
/// [`try_recv`](Self::try_recv) to check whether the user has finished
/// interacting with the dialog.
///
/// # Example
///
/// ```ignore
/// use aurora_platform::dialogs::{FileDialog, PendingDialog};
///
/// // In a click handler — spawns the dialog without blocking the UI:
/// let pending = FileDialog::new()
///     .title("Open File")
///     .open_file_async();
///
/// // Later, check for the result:
/// if let Some(path) = pending.try_recv() {
///     println!("User selected: {:?}", path);
/// }
/// ```
pub struct PendingDialog<T: Send + 'static> {
    result: Arc<Mutex<Option<T>>>,
    handle: Option<JoinHandle<()>>,
}

impl<T: Send + 'static> PendingDialog<T> {
    /// Spawns `f` on a background thread and returns a handle to its result.
    fn spawn(f: impl FnOnce() -> T + Send + 'static) -> Self {
        let result = Arc::new(Mutex::new(None));
        let result_clone = Arc::clone(&result);
        let handle = thread::spawn(move || {
            let value = f();
            if let Ok(mut guard) = result_clone.lock() {
                *guard = Some(value);
            }
        });
        Self {
            result,
            handle: Some(handle),
        }
    }

    /// Returns `true` if the background thread has finished and a result
    /// is available (or the dialog was cancelled / thread panicked).
    pub fn is_ready(&self) -> bool {
        self.handle
            .as_ref()
            .is_none_or(|h| h.is_finished())
    }

    /// Non-blocking check for a result. Returns `Some(T)` exactly once
    /// when the dialog completes, then `None` on all subsequent calls.
    ///
    /// Returns `None` if the thread is still running or if the result
    /// was already consumed by a previous call.
    pub fn try_recv(&self) -> Option<T> {
        if !self.is_ready() {
            return None;
        }
        self.result.lock().ok()?.take()
    }

    /// Blocks the calling thread until the dialog completes, then returns
    /// the result. Consumes `self`.
    ///
    /// Returns `None` if the user cancelled the dialog or if the
    /// background thread panicked.
    pub fn recv(self) -> Option<T> {
        if let Some(handle) = self.handle {
            let _ = handle.join();
        }
        self.result.lock().ok()?.take()
    }
}

/// A file type filter for file dialogs.
///
/// # Example
///
/// ```ignore
/// use aurora_platform::dialogs::FileFilter;
///
/// let filter = FileFilter::new("Images", &["png", "jpg", "gif"]);
/// ```
pub struct FileFilter {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
}

impl FileFilter {
    /// Creates a new file filter with a display name and list of extensions.
    ///
    /// Extensions should not include the leading dot (e.g. `"png"` not `".png"`).
    pub const fn new(name: &'static str, extensions: &'static [&'static str]) -> Self {
        Self { name, extensions }
    }
}

/// A builder for native file dialogs.
///
/// Wraps the platform's native file picker (Open, Save, Folder).
/// All methods consume `self` for chaining.
///
/// # Example
///
/// ```ignore
/// use aurora_platform::dialogs::{FileDialog, FileFilter};
///
/// let path = FileDialog::new()
///     .title("Open Image")
///     .filter(FileFilter::new("Images", &["png", "jpg"]))
///     .filter(FileFilter::new("All Files", &["*"]))
///     .open_file();
/// ```
pub struct FileDialog {
    title: Option<String>,
    default_path: Option<PathBuf>,
    file_name: Option<String>,
    filters: Vec<FileFilter>,
}

impl FileDialog {
    /// Creates a new file dialog builder with no options set.
    pub fn new() -> Self {
        Self {
            title: None,
            default_path: None,
            file_name: None,
            filters: Vec::new(),
        }
    }

    /// Sets the dialog window title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the starting directory for the dialog.
    pub fn default_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.default_path = Some(path.into());
        self
    }

    /// Sets the default file name (for save dialogs).
    pub fn file_name(mut self, name: impl Into<String>) -> Self {
        self.file_name = Some(name.into());
        self
    }

    /// Adds a file type filter.
    pub fn filter(mut self, filter: FileFilter) -> Self {
        self.filters.push(filter);
        self
    }

    // ── Synchronous (blocking) methods ──────────────────────────────

    /// Shows an "Open File" dialog and returns the selected path.
    ///
    /// Returns `None` if the user cancelled.
    pub fn open_file(self) -> Option<PathBuf> {
        self.build_pick().pick_file()
    }

    /// Shows an "Open Files" dialog allowing multiple selection.
    ///
    /// Returns an empty vec if the user cancelled.
    pub fn open_files(self) -> Vec<PathBuf> {
        self.build_pick().pick_files().unwrap_or_default()
    }

    /// Shows a "Save File" dialog and returns the chosen path.
    ///
    /// Returns `None` if the user cancelled.
    pub fn save_file(self) -> Option<PathBuf> {
        self.build_save().save_file()
    }

    /// Shows a "Pick Folder" dialog and returns the selected directory.
    ///
    /// Returns `None` if the user cancelled.
    pub fn pick_folder(self) -> Option<PathBuf> {
        self.build_pick().pick_folder()
    }

    /// Shows an "Open File" dialog parented to the given window.
    ///
    /// The dialog appears as a modal sheet on macOS and a modal dialog on Windows/Linux.
    pub fn open_file_with(self, window: &winit::window::Window) -> Option<PathBuf> {
        self.build_pick().set_parent(window).pick_file()
    }

    /// Shows an "Open Files" dialog parented to the given window.
    pub fn open_files_with(self, window: &winit::window::Window) -> Vec<PathBuf> {
        self.build_pick()
            .set_parent(window)
            .pick_files()
            .unwrap_or_default()
    }

    /// Shows a "Save File" dialog parented to the given window.
    pub fn save_file_with(self, window: &winit::window::Window) -> Option<PathBuf> {
        self.build_save().set_parent(window).save_file()
    }

    /// Shows a "Pick Folder" dialog parented to the given window.
    pub fn pick_folder_with(self, window: &winit::window::Window) -> Option<PathBuf> {
        self.build_pick().set_parent(window).pick_folder()
    }

    // ── Asynchronous (non-blocking) methods ─────────────────────────

    /// Shows an "Open File" dialog on a background thread.
    ///
    /// Returns immediately with a [`PendingDialog`] handle.
    /// The event loop continues while the OS dialog is open.
    pub fn open_file_async(self) -> PendingDialog<Option<PathBuf>> {
        let d = self.build_pick();
        PendingDialog::spawn(move || d.pick_file())
    }

    /// Shows an "Open Files" dialog on a background thread, allowing
    /// multiple selection.
    pub fn open_files_async(self) -> PendingDialog<Vec<PathBuf>> {
        let d = self.build_pick();
        PendingDialog::spawn(move || d.pick_files().unwrap_or_default())
    }

    /// Shows a "Save File" dialog on a background thread.
    pub fn save_file_async(self) -> PendingDialog<Option<PathBuf>> {
        let d = self.build_save();
        PendingDialog::spawn(move || d.save_file())
    }

    /// Shows a "Pick Folder" dialog on a background thread.
    pub fn pick_folder_async(self) -> PendingDialog<Option<PathBuf>> {
        let d = self.build_pick();
        PendingDialog::spawn(move || d.pick_folder())
    }

    /// Shows an "Open File" dialog parented to the given window, on a
    /// background thread.
    ///
    /// The raw window handle is captured before the thread is spawned.
    /// The window must remain alive while the dialog is open (this is
    /// guaranteed when called from within the event loop).
    pub fn open_file_with_async(
        self,
        window: &winit::window::Window,
    ) -> PendingDialog<Option<PathBuf>> {
        let d = self.build_pick_with(window);
        PendingDialog::spawn(move || d.pick_file())
    }

    /// Shows an "Open Files" dialog parented to the given window, on a
    /// background thread.
    pub fn open_files_with_async(
        self,
        window: &winit::window::Window,
    ) -> PendingDialog<Vec<PathBuf>> {
        let d = self.build_pick_with(window);
        PendingDialog::spawn(move || d.pick_files().unwrap_or_default())
    }

    /// Shows a "Save File" dialog parented to the given window, on a
    /// background thread.
    pub fn save_file_with_async(
        self,
        window: &winit::window::Window,
    ) -> PendingDialog<Option<PathBuf>> {
        let d = self.build_save_with(window);
        PendingDialog::spawn(move || d.save_file())
    }

    /// Shows a "Pick Folder" dialog parented to the given window, on a
    /// background thread.
    pub fn pick_folder_with_async(
        self,
        window: &winit::window::Window,
    ) -> PendingDialog<Option<PathBuf>> {
        let d = self.build_pick_with(window);
        PendingDialog::spawn(move || d.pick_folder())
    }

    // ── Internal helpers ────────────────────────────────────────────

    fn apply_common(&self, dialog: rfd::FileDialog) -> rfd::FileDialog {
        let mut d = dialog;
        if let Some(ref title) = self.title {
            d = d.set_title(title);
        }
        if let Some(ref path) = self.default_path {
            d = d.set_directory(path);
        }
        if let Some(ref name) = self.file_name {
            d = d.set_file_name(name);
        }
        for f in &self.filters {
            d = d.add_filter(f.name, f.extensions);
        }
        d
    }

    fn build_pick(self) -> rfd::FileDialog {
        self.apply_common(rfd::FileDialog::new())
    }

    fn build_save(self) -> rfd::FileDialog {
        self.apply_common(rfd::FileDialog::new())
    }

    fn build_pick_with(self, window: &winit::window::Window) -> rfd::FileDialog {
        self.apply_common(rfd::FileDialog::new()).set_parent(window)
    }

    fn build_save_with(self, window: &winit::window::Window) -> rfd::FileDialog {
        self.apply_common(rfd::FileDialog::new()).set_parent(window)
    }
}

impl Default for FileDialog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_sets_fields() {
        let dialog = FileDialog::new()
            .title("Test")
            .default_path("/tmp")
            .file_name("output.txt")
            .filter(FileFilter::new("Text", &["txt"]))
            .filter(FileFilter::new("All", &["*"]));

        assert_eq!(dialog.title.as_deref(), Some("Test"));
        assert_eq!(dialog.default_path, Some(PathBuf::from("/tmp")));
        assert_eq!(dialog.file_name.as_deref(), Some("output.txt"));
        assert_eq!(dialog.filters.len(), 2);
        assert_eq!(dialog.filters[0].name, "Text");
        assert_eq!(dialog.filters[0].extensions, &["txt"]);
        assert_eq!(dialog.filters[1].name, "All");
    }

    #[test]
    fn default_is_empty() {
        let dialog = FileDialog::default();
        assert!(dialog.title.is_none());
        assert!(dialog.default_path.is_none());
        assert!(dialog.file_name.is_none());
        assert!(dialog.filters.is_empty());
    }

    #[test]
    fn pending_dialog_recv_returns_value() {
        let pending = PendingDialog::spawn(|| 42);
        assert_eq!(pending.recv(), Some(42));
    }

    #[test]
    fn pending_dialog_try_recv_once() {
        let pending = PendingDialog::spawn(|| "hello");
        // Wait for the thread to finish
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(pending.is_ready());
        assert_eq!(pending.try_recv(), Some("hello"));
        // Second call returns None (consumed)
        assert_eq!(pending.try_recv(), None);
    }

    #[test]
    fn pending_dialog_is_ready() {
        use std::sync::Barrier;
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = Arc::clone(&barrier);
        let pending = PendingDialog::spawn(move || {
            barrier_clone.wait();
            true
        });
        // Thread is blocked on barrier
        assert!(!pending.is_ready());
        // Release the barrier
        barrier.wait();
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(pending.is_ready());
        assert_eq!(pending.try_recv(), Some(true));
    }

    #[test]
    fn pending_dialog_panicked_thread() {
        let pending: PendingDialog<i32> = PendingDialog::spawn(|| {
            panic!("intentional panic");
        });
        // recv returns None because the result was never written
        assert_eq!(pending.recv(), None);
    }
}
