import {ModalFilterComponent} from "../components/modal/modal-filter.component";
import {ModalDialogComponent} from "../components/modal/modal-dialog.component";
import {ModalImageComponent} from "../components/modal/modal-image.component";
import {BlankComponent} from "../components/modal/blank.component";
import {ModalSpinnerComponent} from "../components/modal/modal-spinner.component";

export interface ModalCommandInterface {
    type: string;
    context?: ModalArgumentsInterface | {};

}


export interface ModalArgumentsInterface {
    title?: string;
    message?: string;
    bigTextBlock?: string;
    status?: string;
    yesButtonText?: string;
    noButtonText?: string;
    lastButtonText?: string;
    tooLastButtonText?: string;
    handler?: Function;
    data?: any;
    locationBack?: boolean;
    idFilterButton?: string;
    filter_data?: ModalFilterArgumentInterface[][];
    position?: ModalPosition;
    style?: ModalStyles;
    type?: string;
}

export interface ModalPosition {
    top?: number;
    bottom?: number;
    right?: number;
    left?: number;
    height?: number;
    width?: number;
}

export interface ModalStyles {
    isMinContent?: boolean;
    width?: string;
    borderRadius?: string;
}

export interface ModalFilterArgumentInterface {
    value: string;
    title: string;
    active: boolean;
}

export interface ModalMappingsComponent {
    filter: typeof ModalFilterComponent;
    dialog: typeof ModalDialogComponent;
    image: typeof ModalImageComponent;
    blank: typeof BlankComponent;
    spinner: typeof ModalSpinnerComponent;
}