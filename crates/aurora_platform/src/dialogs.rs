use std::path::PathBuf;

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
}
