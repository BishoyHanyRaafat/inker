import { BrowserRouter, Routes, Route } from "react-router-dom";
import Home from "./pages/HomePage";
import Login from "./pages/LoginPage";
import Signup from "./pages/SignupPage";
import Dashboard from "./pages/DashboardPage";
import YTInteractive from "./pages/YTInteractivePage";
import NoteDetail from "./pages/NoteDetailPage";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/login" element={<Login />} />
        <Route path="/signup" element={<Signup />} />
        <Route path="/dashboard" element={<Dashboard />} />
        <Route path="/interactive/yt" element={<YTInteractive />} />
        <Route path="/notes/:id" element={<NoteDetail />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
