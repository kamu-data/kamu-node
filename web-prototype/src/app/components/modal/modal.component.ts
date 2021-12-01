import {
    Component, OnInit,
    ViewChild, ViewContainerRef,
    ComponentFactoryResolver,
    ComponentRef
} from '@angular/core';


import { BlankComponent} from './blank.component';
import { DynamicComponent} from './dynamic.component';
import { ModalCommandInterface } from '../../interface/modal.interface';
import { ModalDialogComponent} from './modal-dialog.component';
import { ModalImageComponent } from './modal-image.component';
import { ModalService } from './modal.service';
import { ModalSpinnerComponent} from './modal-spinner.component';
import {Location} from '@angular/common';
import {ModalFilterComponent} from './modal-filter.component';
import {ModalMappingsComponent} from '../../interface/modal.interface';


@Component({
    selector:   'modal',
    template:   `
        <div class="modal__container" #container tabindex="1"
                [ngClass]="{'modal__invisible': !isVisible}">
        </div>
    `
})


export class ModalComponent implements OnInit {
    @ViewChild('container', { read: ViewContainerRef })

    container: ViewContainerRef;
    isVisible: boolean;

    private componentRef: ComponentRef<unknown>;
    private mappings: ModalMappingsComponent = {
        blank:                BlankComponent,
        dialog:               ModalDialogComponent,
        image:                ModalImageComponent,
        spinner:              ModalSpinnerComponent,
        filter:               ModalFilterComponent,
    };


    constructor(
        private componentFactoryResolver: ComponentFactoryResolver,
        private modalService: ModalService,
        private location: Location
    ) { }



    ngOnInit() {
        this.modalService.getCommand()
            .subscribe((command: ModalCommandInterface) => {
                this._execute(command);
            });
    }



    _execute(command: ModalCommandInterface) {
        this.modalService.modalType = command.type;

        this._close();

        if (command.type && command.type !== 'blank') {
            this._renderModal(command);
        }
    }



    _renderModal(command: ModalCommandInterface) {
        const componentType   = this._getComponentType(command.type);

        const factory         = this.componentFactoryResolver.resolveComponentFactory(componentType);
        this.componentRef   = this.container.createComponent(factory);

        const instance        = this.componentRef.instance as DynamicComponent;
        instance.context    = Object.assign(command.context, {_close: this._close.bind(this)});

        this._handleKBD(command.type);
    }



    _getComponentType(typeName: string) {
        // @ts-ignore
        const type = this.mappings[typeName];
        return type || BlankComponent;
    }



    _handleKBD(type: string) {
        this.isVisible = true;
        document.addEventListener('keydown', this._processKDB.bind(this));
    }



    _close(location?: boolean) {
        if (location === true) { this.location.back(); }

        this.isVisible = false;
        this.container.remove();

        document.removeEventListener('keydown', this._processKDB.bind(this));
    }



    _processKDB(e: any) {
        // if (e.keyCode === 27) { // escape
        //     this._close();
        // }

        if (e.keyCode === 9) { // tab
            setTimeout(() => this.container.element.nativeElement.focus());
        }
    }

}
