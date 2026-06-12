# Friends of `yame`

While `yame` works perfectly fine on its own, it integrates with several optional terminal tools.

---

## 1. `fd` and `fzf` (Fuzzy Search & File Opening)

### Why you want this:
`yame` can use `fd` (a fast directory search tool) and `fzf` (a command-line fuzzy finder). 

When these tools are installed:

* Running `yame` with no arguments opens a list of all Markdown files in the current folder.
* Typing `yame my-file` searches for files containing "my-file" and opens the closest match.
* You can fuzzy-filter through your files interactively inside the terminal.

### How to install:

#### macOS (Homebrew)

```sh
brew install fd fzf
```

#### Linux (Debian/Ubuntu)

```sh
sudo apt install fd-find fzf
```

*(Note: Debian-based systems install `fd` as `fdfind`. The shell integration automatically handles this mapping.)*

#### Windows (Winget)

```powershell
winget install sharkdp.fd
winget install junegunn.fzf
```

---

## 2. `lf` (File Manager)

### Why you want this:

`lf` (List Files) is a fast, terminal-based file manager. Since `yame` does not include a built-in file tree or directory sidebar, running `lf` as a file manager allows `yame` to browse and open files interactively.

### How to install:

#### macOS (Homebrew)

```sh
brew install lf
```

#### Linux (Debian/Ubuntu)

```sh
sudo apt install lf
```

#### Windows (Winget)

```powershell
winget install gokcehan.lf
```

---

## 3. `bat` (Syntax-Highlighting File Previewer)

### Why you want this:

`bat` is a clone of the classic `cat` command that supports syntax highlighting, Git modifications, and automatic paging. `yame` uses it with `lf` to make the file previews while you're browsing pretty!

### How to install:

#### macOS (Homebrew)
```sh
brew install bat
```

#### Linux (Debian/Ubuntu)
```sh
sudo apt install bat
```

#### Windows (Winget)
```powershell
winget install sharkdp.bat
```
