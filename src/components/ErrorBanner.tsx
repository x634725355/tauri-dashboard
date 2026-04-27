export default function ErrorBanner({ message }: { message: string }) {
  return (
    <div
      className="mb-3.5 rounded-xl border border-error-200 bg-error-50 px-3 py-2.5 text-sm text-error-800
                 dark:border-error-800 dark:bg-error-900/30 dark:text-error-200"
      role="alert"
    >
      {message}
    </div>
  );
}
