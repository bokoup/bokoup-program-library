import { createTheme, StyledEngineProvider, ThemeProvider } from '@mui/material';
import { deepPurple } from '@mui/material/colors';
import { SnackbarProvider } from 'notistack';
import shadows, { Shadows } from '@mui/material/styles/shadows';
import { FC, ReactNode } from 'react';

const theme = createTheme({
    shadows: shadows.map(() => 'none') as Shadows,
    typography: {
        fontSize: 18,
        fontFamily: 'Assistant, sans-serif'
    },
    palette: {
        primary: {
            main: deepPurple[700],
        },
    },
    components: {
        MuiButtonBase: {
            styleOverrides: {
                root: {
                    justifyContent: 'flex-start',
                },
            },
        },
        MuiButton: {
            styleOverrides: {
                root: {
                    textTransform: 'none',
                    padding: '12px 16px',
                    borderRadius: 0
                },
                startIcon: {
                    marginRight: 8,
                },
                endIcon: {
                    marginLeft: 8,
                },
            },
        },
        MuiDialogTitle: {
            styleOverrides: {
                root: {
                    color: 'white',
                },
            },
        },
        MuiDialog: {
            styleOverrides: {
                paper: {
                    borderRadius: 0,
                },
            },
        },
        MuiDialogContent: {
            styleOverrides: {
                root: {
                    backgroundColor: 'inherit',
                    fontSize: 15,
                }
            },
        }
    },
});

export const Theme: FC<{ children: ReactNode }> = ({ children }) => {
    return (
        <StyledEngineProvider injectFirst>
            <ThemeProvider theme={theme}>
                <SnackbarProvider>{children}</SnackbarProvider>
            </ThemeProvider>
        </StyledEngineProvider>
    );
};
