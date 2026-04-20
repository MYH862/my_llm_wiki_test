import { useState } from "react"
import { LoginPage } from "./login-page"
import { RegisterPage } from "./register-page"

export function AuthRouter() {
  const [isLogin, setIsLogin] = useState(true)

  if (isLogin) {
    return <LoginPage onSwitchToRegister={() => setIsLogin(false)} />
  }

  return <RegisterPage onSwitchToLogin={() => setIsLogin(true)} />
}
