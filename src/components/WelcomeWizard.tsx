import { useState, useEffect } from 'react';
import { 
  verifyBundledExecutables,
  selectDirectory, 
  saveSettings
} from '../api/tauri';
import type { AppSettings } from '../types';

interface WelcomeWizardProps {
  settings: AppSettings;
  onComplete: (settings: AppSettings) => void;
}

type WizardStep = 'welcome' | 'verification' | 'savePath' | 'tutorial';

export default function WelcomeWizard({ settings, onComplete }: WelcomeWizardProps) {
  const [currentStep, setCurrentStep] = useState<WizardStep>('welcome');
  const [isVerifying, setIsVerifying] = useState(false);
  const [verificationPassed, setVerificationPassed] = useState<boolean | null>(null);
  const [savePath, setSavePath] = useState(settings.defaultSavePath || '');

  // Verify bundled executables when reaching that step
  useEffect(() => {
    if (currentStep === 'verification') {
      verifyExecutables();
    }
  }, [currentStep]);

  const verifyExecutables = async () => {
    setIsVerifying(true);
    setVerificationPassed(null);
    
    try {
      const result = await verifyBundledExecutables();
      setVerificationPassed(result);
      
      // If verification passed, automatically move to next step after a brief delay
      if (result) {
        setTimeout(() => {
          setCurrentStep('savePath');
        }, 1500);
      }
    } catch (error) {
      console.error('Failed to verify bundled executables:', error);
      setVerificationPassed(false);
    } finally {
      setIsVerifying(false);
    }
  };

  const handleSelectDirectory = async () => {
    try {
      const selected = await selectDirectory();
      if (selected) {
        setSavePath(selected);
      }
    } catch (error) {
      console.error('Failed to select directory:', error);
    }
  };

  const handleComplete = async () => {
    const updatedSettings: AppSettings = {
      ...settings,
      defaultSavePath: savePath,
      firstLaunchCompleted: true
    };
    
    try {
      console.log('[WelcomeWizard] Saving settings with firstLaunchCompleted=true:', updatedSettings);
      await saveSettings(updatedSettings);
      console.log('[WelcomeWizard] Settings saved successfully');
      onComplete(updatedSettings);
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  };

  const steps = [
    { id: 'welcome', label: 'Welcome', number: 0 },
    { id: 'verification', label: 'Verification', number: 1 },
    { id: 'savePath', label: 'Location', number: 2 },
    { id: 'tutorial', label: 'Tutorial', number: 3 },
  ];

  const currentStepIndex = steps.findIndex(s => s.id === currentStep);

  return (
    <div className="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50 p-4">
      <div className="bg-gray-800 rounded-xl shadow-2xl max-w-2xl w-full max-h-[90vh] overflow-y-auto">
        {/* Step Indicator */}
        {currentStep !== 'welcome' && (
          <div className="px-8 pt-6 pb-4 border-b border-gray-700">
            <div className="flex items-center justify-between">
              {steps.slice(1).map((step, index) => (
                <div key={step.id} className="flex items-center flex-1">
                  <div className="flex flex-col items-center flex-1">
                    <div
                      className={`w-10 h-10 rounded-full flex items-center justify-center font-semibold text-sm transition-all duration-300 ${
                        currentStepIndex >= index + 1
                          ? 'bg-blue-600 text-white'
                          : 'bg-gray-700 text-gray-400'
                      }`}
                    >
                      {currentStepIndex > index + 1 ? (
                        <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
                        </svg>
                      ) : (
                        step.number
                      )}
                    </div>
                    <span
                      className={`mt-2 text-xs font-medium transition-colors duration-300 ${
                        currentStepIndex >= index + 1 ? 'text-white' : 'text-gray-500'
                      }`}
                    >
                      {step.label}
                    </span>
                  </div>
                  {index < steps.length - 2 && (
                    <div
                      className={`h-0.5 flex-1 mx-2 transition-colors duration-300 ${
                        currentStepIndex > index + 1 ? 'bg-blue-600' : 'bg-gray-700'
                      }`}
                    />
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Welcome Step */}
        {currentStep === 'welcome' && (
          <div className="p-8 animate-fade-in">
            <div className="text-center mb-10">
              <div className="w-24 h-24 bg-gradient-to-br from-blue-600 to-blue-700 rounded-full flex items-center justify-center mx-auto mb-6 shadow-lg animate-bounce-in">
                <svg className="w-14 h-14 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <h1 className="text-3xl font-bold text-white mb-3">Welcome to YouTube Downloader</h1>
              <p className="text-gray-400 text-lg">Let's get you set up in just a few steps</p>
            </div>

            <div className="space-y-5 mb-10">
              <div className="flex items-start gap-4 p-4 rounded-lg bg-gray-700 bg-opacity-50 hover:bg-opacity-70 transition-all duration-200">
                <div className="flex-shrink-0 w-10 h-10 bg-blue-600 rounded-full flex items-center justify-center text-white font-bold shadow-md">1</div>
                <div className="flex-1">
                  <h3 className="text-white font-semibold mb-1">Verify Installation</h3>
                  <p className="text-gray-400 text-sm leading-relaxed">Check bundled executables integrity</p>
                </div>
              </div>
              <div className="flex items-start gap-4 p-4 rounded-lg bg-gray-700 bg-opacity-50 hover:bg-opacity-70 transition-all duration-200">
                <div className="flex-shrink-0 w-10 h-10 bg-blue-600 rounded-full flex items-center justify-center text-white font-bold shadow-md">2</div>
                <div className="flex-1">
                  <h3 className="text-white font-semibold mb-1">Set Download Location</h3>
                  <p className="text-gray-400 text-sm leading-relaxed">Choose where to save your videos</p>
                </div>
              </div>
              <div className="flex items-start gap-4 p-4 rounded-lg bg-gray-700 bg-opacity-50 hover:bg-opacity-70 transition-all duration-200">
                <div className="flex-shrink-0 w-10 h-10 bg-blue-600 rounded-full flex items-center justify-center text-white font-bold shadow-md">3</div>
                <div className="flex-1">
                  <h3 className="text-white font-semibold mb-1">Quick Tutorial</h3>
                  <p className="text-gray-400 text-sm leading-relaxed">Learn the basics</p>
                </div>
              </div>
            </div>

            <button
              onClick={() => setCurrentStep('verification')}
              className="w-full bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-700 hover:to-blue-800 text-white font-semibold py-4 px-6 rounded-lg transition-all duration-200 shadow-lg hover:shadow-xl transform hover:scale-[1.02]"
            >
              Get Started
            </button>
          </div>
        )}

        {/* Verification Step */}
        {currentStep === 'verification' && (
          <div className="p-8 animate-fade-in">
            <div className="text-center mb-6">
              <h2 className="text-2xl font-bold text-white mb-2">Verifying Installation</h2>
              <p className="text-gray-400">
                Checking the integrity of bundled executables...
              </p>
            </div>

            <div className="flex flex-col items-center justify-center py-16">
              {isVerifying && (
                <div className="animate-fade-in">
                  <div className="relative">
                    <div className="animate-spin rounded-full h-20 w-20 border-4 border-gray-700 border-t-blue-600 mb-8"></div>
                    <div className="absolute inset-0 flex items-center justify-center">
                      <div className="w-12 h-12 bg-blue-600 rounded-full opacity-20 animate-pulse"></div>
                    </div>
                  </div>
                  <p className="text-gray-300 text-lg font-medium">Verifying bundled executables...</p>
                  <p className="text-gray-500 text-sm mt-2">This will only take a moment</p>
                </div>
              )}

              {!isVerifying && verificationPassed === true && (
                <div className="animate-scale-in text-center">
                  <div className="relative mb-8">
                    <div className="w-20 h-20 bg-gradient-to-br from-green-500 to-green-600 rounded-full flex items-center justify-center shadow-lg animate-pulse-success">
                      <svg className="w-12 h-12 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
                      </svg>
                    </div>
                  </div>
                  <p className="text-green-400 text-xl font-bold mb-2">Verification Successful!</p>
                  <p className="text-gray-400">All bundled executables are intact and ready to use.</p>
                  <p className="text-gray-500 text-sm mt-4">Proceeding to next step...</p>
                </div>
              )}

              {!isVerifying && verificationPassed === false && (
                <div className="animate-scale-in w-full max-w-lg">
                  <div className="text-center mb-8">
                    <div className="w-20 h-20 bg-gradient-to-br from-red-500 to-red-600 rounded-full flex items-center justify-center mx-auto mb-6 shadow-lg">
                      <svg className="w-12 h-12 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </div>
                    <p className="text-red-400 text-xl font-bold mb-2">Verification Failed</p>
                    <p className="text-gray-400 mb-8">
                      The bundled executables could not be verified. This may indicate a corrupted installation.
                    </p>
                  </div>
                  
                  <div className="p-5 bg-red-900 bg-opacity-30 border border-red-600 rounded-lg mb-8">
                    <div className="flex items-start gap-4">
                      <svg className="w-7 h-7 text-red-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                      </svg>
                      <div>
                        <h4 className="text-white font-semibold mb-2">Action Required</h4>
                        <p className="text-gray-300 text-sm leading-relaxed">
                          Please reinstall the application to fix this issue. Download the latest version from the official website.
                        </p>
                      </div>
                    </div>
                  </div>

                  <div className="flex gap-4">
                    <button
                      onClick={() => setCurrentStep('welcome')}
                      className="flex-1 bg-gray-700 hover:bg-gray-600 text-white font-semibold py-3 px-6 rounded-lg transition-all duration-200"
                    >
                      Back
                    </button>
                    <button
                      onClick={verifyExecutables}
                      className="flex-1 bg-blue-600 hover:bg-blue-700 text-white font-semibold py-3 px-6 rounded-lg transition-all duration-200 shadow-lg hover:shadow-xl"
                    >
                      Retry Verification
                    </button>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Save Path Step */}
        {currentStep === 'savePath' && (
          <div className="p-8 animate-fade-in">
            <div className="text-center mb-8">
              <div className="w-16 h-16 bg-gradient-to-br from-blue-600 to-blue-700 rounded-full flex items-center justify-center mx-auto mb-4 shadow-lg">
                <svg className="w-9 h-9 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
              </div>
              <h2 className="text-2xl font-bold text-white mb-2">Set Download Location</h2>
              <p className="text-gray-400">
                Choose where your downloaded videos will be saved
              </p>
            </div>

            <div className="mb-8">
              <label className="block text-sm font-medium text-gray-300 mb-3">
                Download Folder
              </label>
              <div className="flex gap-3">
                <input
                  type="text"
                  value={savePath}
                  readOnly
                  placeholder="No folder selected"
                  className="flex-1 bg-gray-700 border border-gray-600 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent transition-all duration-200"
                />
                <button
                  onClick={handleSelectDirectory}
                  className="bg-blue-600 hover:bg-blue-700 text-white font-semibold px-8 py-3 rounded-lg transition-all duration-200 shadow-lg hover:shadow-xl flex items-center gap-2"
                >
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                  </svg>
                  Browse
                </button>
              </div>
              {savePath ? (
                <div className="mt-4 p-4 bg-green-900 bg-opacity-20 border border-green-600 rounded-lg">
                  <div className="flex items-start gap-3">
                    <svg className="w-5 h-5 text-green-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    <div>
                      <p className="text-sm text-green-400 font-medium mb-1">Folder Selected</p>
                      <p className="text-sm text-gray-300 break-all">{savePath}</p>
                    </div>
                  </div>
                </div>
              ) : (
                <div className="mt-4 p-4 bg-blue-900 bg-opacity-20 border border-blue-600 rounded-lg">
                  <div className="flex items-start gap-3">
                    <svg className="w-5 h-5 text-blue-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    <p className="text-sm text-gray-300">
                      Click "Browse" to select a folder. You can change this later in settings.
                    </p>
                  </div>
                </div>
              )}
            </div>

            <div className="flex gap-4">
              <button
                onClick={() => setCurrentStep('welcome')}
                className="flex-1 bg-gray-700 hover:bg-gray-600 text-white font-semibold py-3 px-6 rounded-lg transition-all duration-200"
              >
                Back
              </button>
              <button
                onClick={() => setCurrentStep('tutorial')}
                disabled={!savePath}
                className="flex-1 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-700 hover:to-blue-800 text-white font-semibold py-3 px-6 rounded-lg transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed shadow-lg hover:shadow-xl disabled:shadow-none"
              >
                Continue
              </button>
            </div>
          </div>
        )}

        {/* Tutorial Step */}
        {currentStep === 'tutorial' && (
          <div className="p-8 animate-fade-in">
            <div className="text-center mb-8">
              <div className="w-16 h-16 bg-gradient-to-br from-blue-600 to-blue-700 rounded-full flex items-center justify-center mx-auto mb-4 shadow-lg">
                <svg className="w-9 h-9 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
                </svg>
              </div>
              <h2 className="text-2xl font-bold text-white mb-2">Quick Start Guide</h2>
              <p className="text-gray-400">
                Here's how to use YouTube Downloader
              </p>
            </div>

            <div className="space-y-4 mb-8">
              <div className="flex items-start gap-4 p-5 rounded-lg bg-gray-700 bg-opacity-50 hover:bg-opacity-70 transition-all duration-200">
                <div className="flex-shrink-0 w-12 h-12 bg-gradient-to-br from-blue-600 to-blue-700 rounded-lg flex items-center justify-center text-white font-bold shadow-md">
                  1
                </div>
                <div className="flex-1">
                  <h3 className="text-white font-semibold mb-2">Paste a YouTube URL</h3>
                  <p className="text-gray-400 text-sm leading-relaxed">
                    Copy any YouTube video, playlist, or channel URL and paste it into the input field.
                  </p>
                </div>
              </div>

              <div className="flex items-start gap-4 p-5 rounded-lg bg-gray-700 bg-opacity-50 hover:bg-opacity-70 transition-all duration-200">
                <div className="flex-shrink-0 w-12 h-12 bg-gradient-to-br from-blue-600 to-blue-700 rounded-lg flex items-center justify-center text-white font-bold shadow-md">
                  2
                </div>
                <div className="flex-1">
                  <h3 className="text-white font-semibold mb-2">Preview and Select</h3>
                  <p className="text-gray-400 text-sm leading-relaxed">
                    Review the video information, choose quality settings, and select which videos to download.
                  </p>
                </div>
              </div>

              <div className="flex items-start gap-4 p-5 rounded-lg bg-gray-700 bg-opacity-50 hover:bg-opacity-70 transition-all duration-200">
                <div className="flex-shrink-0 w-12 h-12 bg-gradient-to-br from-blue-600 to-blue-700 rounded-lg flex items-center justify-center text-white font-bold shadow-md">
                  3
                </div>
                <div className="flex-1">
                  <h3 className="text-white font-semibold mb-2">Add to Queue</h3>
                  <p className="text-gray-400 text-sm leading-relaxed">
                    Click "Add to Queue" to start downloading. Monitor progress in the download queue panel.
                  </p>
                </div>
              </div>

              <div className="flex items-start gap-4 p-5 rounded-lg bg-gray-700 bg-opacity-50 hover:bg-opacity-70 transition-all duration-200">
                <div className="flex-shrink-0 w-12 h-12 bg-gradient-to-br from-blue-600 to-blue-700 rounded-lg flex items-center justify-center text-white font-bold shadow-md">
                  4
                </div>
                <div className="flex-1">
                  <h3 className="text-white font-semibold mb-2">Manage Downloads</h3>
                  <p className="text-gray-400 text-sm leading-relaxed">
                    Pause, resume, or cancel downloads. Reorder the queue by dragging items.
                  </p>
                </div>
              </div>
            </div>

            <div className="bg-gradient-to-r from-blue-900 to-blue-800 bg-opacity-40 border border-blue-600 rounded-lg p-5 mb-8 shadow-lg">
              <div className="flex items-start gap-4">
                <div className="flex-shrink-0 w-10 h-10 bg-blue-600 rounded-full flex items-center justify-center shadow-md">
                  <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                </div>
                <div className="flex-1">
                  <h4 className="text-white font-semibold mb-2">Pro Tip</h4>
                  <p className="text-gray-300 text-sm leading-relaxed">
                    You can download entire channels organized by playlists. The app will automatically create folders to keep everything organized!
                  </p>
                </div>
              </div>
            </div>

            <div className="flex gap-4">
              <button
                onClick={() => setCurrentStep('savePath')}
                className="flex-1 bg-gray-700 hover:bg-gray-600 text-white font-semibold py-3 px-6 rounded-lg transition-all duration-200"
              >
                Back
              </button>
              <button
                onClick={handleComplete}
                className="flex-1 bg-gradient-to-r from-green-600 to-green-700 hover:from-green-700 hover:to-green-800 text-white font-semibold py-3 px-6 rounded-lg transition-all duration-200 shadow-lg hover:shadow-xl flex items-center justify-center gap-2"
              >
                <span>Start Using App</span>
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
                </svg>
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
