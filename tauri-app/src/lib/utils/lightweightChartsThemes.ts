import { ColorType } from 'lightweight-charts';

export const themes = {
  dark: {
    layout: {
      textColor: 'white',
      background: { type: ColorType.Solid, color: "#1F2937" },
    },
    grid: {
      vertLines: { color: '#444' },
      horzLines: { color: '#444' },
    },
  },
  light: {
    layout: {
      textColor: 'black',
      background: { type: ColorType.Solid, color: 'white' },
    },
    grid: {
      vertLines: { color: '#444' },
      horzLines: { color: '#444' },
    },
  }
}