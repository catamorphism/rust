(setq load-path (cons "." load-path))
(defun rustmode-compile () (mapcar (lambda (x) (byte-compile-file x))))
(list "cm-mode.el" "rust-mode.el")
