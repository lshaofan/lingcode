import { ToastProvider } from './components'
import { MainWindow } from './windows/main/MainWindow'

function App() {
  return (
    <ToastProvider>
      <MainWindow />
    </ToastProvider>
  )
}

export default App
