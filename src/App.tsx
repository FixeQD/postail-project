import { useState } from 'react'
import reactLogo from './assets/react.svg'
import { invoke } from '@tauri-apps/api/core'
import { TitleBar } from './components/TitleBar'

function App() {
	const [greetMsg, setGreetMsg] = useState('')
	const [name, setName] = useState('')

	async function greet() {
		setGreetMsg(await invoke('greet', { name }))
	}

	return (
		<div className='flex h-screen flex-col bg-gray-900'>
			<TitleBar />

			{/* Main Content */}
			<main className='flex flex-1 flex-col items-center justify-center overflow-auto text-center text-gray-100'>
				<h1 className='mb-6 text-2xl font-bold'>Welcome to Tauri + React</h1>

				<div className='mb-4 flex justify-center'>
					<a
						href='https://vite.dev'
						target='_blank'
						className='font-medium text-blue-400 hover:text-blue-300'>
						<img
							src='/vite.svg'
							className='h-24 p-6 transition-all duration-700 hover:drop-shadow-[0_0_2em_#747bff]'
							alt='Vite logo'
						/>
					</a>
					<a
						href='https://tauri.app'
						target='_blank'
						className='font-medium text-blue-400 hover:text-blue-300'>
						<img
							src='/tauri.svg'
							className='h-24 p-6 transition-all duration-700 hover:drop-shadow-[0_0_2em_#24c8db]'
							alt='Tauri logo'
						/>
					</a>
					<a
						href='https://react.dev'
						target='_blank'
						className='font-medium text-blue-400 hover:text-blue-300'>
						<img
							src={reactLogo}
							className='h-24 p-6 transition-all duration-700 hover:drop-shadow-[0_0_2em_#61dafb]'
							alt='React logo'
						/>
					</a>
				</div>

				<p className='mb-6 text-gray-400'>
					Click on the Tauri, Vite, and React logos to learn more.
				</p>

				<form
					className='flex justify-center gap-2'
					onSubmit={(e) => {
						e.preventDefault()
						greet()
					}}>
					<input
						id='greet-input'
						className='rounded-lg border border-gray-700 bg-gray-800 px-4 py-2 text-gray-100 placeholder-gray-500 transition-all duration-200 outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500'
						onChange={(e) => setName(e.currentTarget.value)}
						placeholder='Enter a name...'
					/>
					<button
						className='rounded-lg bg-blue-600 px-4 py-2 font-medium text-white transition-colors duration-150 hover:bg-blue-500 active:bg-blue-700'
						type='submit'>
						Greet
					</button>
				</form>

				{greetMsg && <p className='mt-4 text-lg text-gray-300'>{greetMsg}</p>}
			</main>
		</div>
	)
}

export default App
