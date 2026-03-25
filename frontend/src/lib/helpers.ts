import { Role, WasteType } from '../api/types';

export const formatAddress = (addr: string): string =>
  `${addr.slice(0, 4)}...${addr.slice(-4)}`;

export const formatTokenAmount = (amount: bigint | number, decimals = 7): string => {
  const num = Number(amount) / 10 ** decimals;
  return num.toLocaleString(undefined, { maximumFractionDigits: decimals });
};

export const wasteTypeLabel = (type: WasteType): string =>
  ({ [WasteType.Paper]: 'Paper', [WasteType.PetPlastic]: 'PET Plastic', [WasteType.Plastic]: 'Plastic', [WasteType.Metal]: 'Metal', [WasteType.Glass]: 'Glass' })[type] ?? 'Unknown';

export const roleLabel = (role: Role): string =>
  ({ [Role.Recycler]: 'Recycler', [Role.Collector]: 'Collector', [Role.Manufacturer]: 'Manufacturer' })[role] ?? 'Unknown';

export const formatDate = (timestamp: number): string =>
  new Date(timestamp * 1000).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
