import { useState } from 'react'
import { Button, Input, Modal, ConfirmModal, useToast } from './components'

export function ComponentsDemo() {
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [isConfirmOpen, setIsConfirmOpen] = useState(false)
  const [inputValue, setInputValue] = useState('')
  const [inputError, setInputError] = useState('')
  const toast = useToast()

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value
    setInputValue(value)

    if (value.length < 3 && value.length > 0) {
      setInputError('至少需要 3 个字符')
    } else {
      setInputError('')
    }
  }

  return (
    <div className="min-h-screen bg-gray-50 p-8">
      <div className="max-w-4xl mx-auto space-y-8">
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-2xl font-bold mb-6">组件库演示</h2>

          {/* Button 演示 */}
          <section className="mb-8">
            <h3 className="text-lg font-semibold mb-4">Button 按钮</h3>
            <div className="space-y-4">
              <div className="flex flex-wrap gap-3">
                <Button variant="primary">Primary</Button>
                <Button variant="secondary">Secondary</Button>
                <Button variant="ghost">Ghost</Button>
                <Button variant="danger">Danger</Button>
              </div>
              <div className="flex flex-wrap gap-3">
                <Button size="sm">Small</Button>
                <Button size="md">Medium</Button>
                <Button size="lg">Large</Button>
              </div>
              <div className="flex flex-wrap gap-3">
                <Button isLoading>Loading</Button>
                <Button isDisabled>Disabled</Button>
              </div>
            </div>
          </section>

          {/* Input 演示 */}
          <section className="mb-8">
            <h3 className="text-lg font-semibold mb-4">Input 输入框</h3>
            <div className="space-y-4 max-w-md">
              <Input label="用户名" placeholder="请输入用户名" />
              <Input
                type="password"
                label="密码"
                placeholder="请输入密码"
                helperText="密码长度至少 8 位"
              />
              <Input
                label="验证输入"
                value={inputValue}
                onChange={handleInputChange}
                error={inputError}
                placeholder="输入至少 3 个字符"
              />
              <Input label="禁用状态" disabled value="禁用的输入框" />
            </div>
          </section>

          {/* Modal 演示 */}
          <section className="mb-8">
            <h3 className="text-lg font-semibold mb-4">Modal 对话框</h3>
            <div className="flex flex-wrap gap-3">
              <Button onClick={() => setIsModalOpen(true)}>打开 Modal</Button>
              <Button onClick={() => setIsConfirmOpen(true)} variant="danger">
                打开确认对话框
              </Button>
            </div>
          </section>

          {/* Toast 演示 */}
          <section>
            <h3 className="text-lg font-semibold mb-4">Toast 通知</h3>
            <div className="flex flex-wrap gap-3">
              <Button onClick={() => toast.success('操作成功！')} variant="primary">
                成功通知
              </Button>
              <Button onClick={() => toast.error('操作失败！')} variant="danger">
                错误通知
              </Button>
              <Button onClick={() => toast.warning('警告信息')} variant="secondary">
                警告通知
              </Button>
              <Button onClick={() => toast.info('提示信息')} variant="ghost">
                信息通知
              </Button>
            </div>
          </section>
        </div>
      </div>

      {/* Modals */}
      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title="示例对话框"
        size="md"
        footer={
          <div className="flex justify-end gap-2">
            <Button variant="ghost" onClick={() => setIsModalOpen(false)}>
              取消
            </Button>
            <Button onClick={() => setIsModalOpen(false)}>确定</Button>
          </div>
        }
      >
        <div className="space-y-4">
          <p>这是一个示例对话框的内容。</p>
          <p>你可以在这里放置任何内容，包括表单、列表等。</p>
          <Input label="示例输入" placeholder="在对话框中输入..." />
        </div>
      </Modal>

      <ConfirmModal
        isOpen={isConfirmOpen}
        onClose={() => setIsConfirmOpen(false)}
        title="确认操作"
        message="你确定要执行这个操作吗？此操作无法撤销。"
        confirmText="确认删除"
        cancelText="取消"
        confirmVariant="danger"
        onConfirm={() => {
          toast.success('操作已确认')
          setIsConfirmOpen(false)
        }}
      />
    </div>
  )
}
