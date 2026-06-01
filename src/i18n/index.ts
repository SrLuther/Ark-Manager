import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import LanguageDetector from 'i18next-browser-languagedetector'
import ptBR from './locales/pt-BR.json'

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    lng: 'pt-BR',
    fallbackLng: 'pt-BR',
    resources: {
      'pt-BR': { translation: ptBR },
    },
    interpolation: {
      escapeValue: false,
    },
  })

export default i18n
