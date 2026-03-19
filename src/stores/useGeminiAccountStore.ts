import {
  GeminiAccount,
  getGeminiAccountDisplayEmail,
  getGeminiPlanBadge,
  getGeminiUsage,
} from '../types/gemini';
import * as geminiService from '../services/geminiService';
import { createProviderAccountStore } from './createProviderAccountStore';

const GEMINI_ACCOUNTS_CACHE_KEY = 'agtools.gemini.accounts.cache';

export const useGeminiAccountStore = createProviderAccountStore<GeminiAccount>(
  GEMINI_ACCOUNTS_CACHE_KEY,
  {
    listAccounts: geminiService.listGeminiAccounts,
    deleteAccount: geminiService.deleteGeminiAccount,
    deleteAccounts: geminiService.deleteGeminiAccounts,
    injectAccount: geminiService.injectGeminiAccount,
    refreshToken: geminiService.refreshGeminiToken,
    refreshAllTokens: geminiService.refreshAllGeminiTokens,
    importFromJson: geminiService.importGeminiFromJson,
    exportAccounts: geminiService.exportGeminiAccounts,
    updateAccountTags: geminiService.updateGeminiAccountTags,
  },
  {
    getDisplayEmail: getGeminiAccountDisplayEmail,
    getPlanBadge: getGeminiPlanBadge,
    getUsage: (account) => {
      const usage = getGeminiUsage(account);
      return {
        inlineSuggestionsUsedPercent: usage.totalPercentUsed,
        chatMessagesUsedPercent: usage.totalPercentUsed,
      };
    },
  },
);
