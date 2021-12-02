import { Injectable } from '@angular/core';
import {
    ModalArgumentsInterface,
    ModalCommandInterface
} from '../../interface/modal.interface';
import {Subject} from 'rxjs';



@Injectable()
export class ModalService {
    /**
     * @example
     * - show an image:
     * this.modalService.showImage("https://bizibazapics.s3.amazonaws.com/SA1/148136152005080520165418.jpg");
     *
     * - close the modal window
     * this.modalService.close();
     *
     * - show warning and handle users choice
     * this.modalService.
     *      .warning({
     *           title:          'Hello!',
     *           message:        'Do you like this modal?',
     *           yesButtonText:  'Yes',
     *           noButtonText:   'Probably',
     *       })
     * .then(action => console.log('User said: ', action));
     */

    private showModal$: Subject<ModalCommandInterface> = new Subject<ModalCommandInterface>();
    private currentModalType = 'blank';


    /**
     * Setter for type of currently displayed modal.
     */
    set modalType(type: string) {
        this.currentModalType = type;
    }


    /**
     * Getter for type of currently displayed modal.
     */
    get modalType(): string {
        return this.currentModalType;
    }



    public close(): void {
        this.showModal$.next({
            context:    {},
            buttonCount: 0,
            type:       'blank'
        });
    }



    public success(options: ModalArgumentsInterface): Promise<{}> {
        return this._showDialog( Object.assign(options, { status: 'ok'}) );
    }



    public warning(options: ModalArgumentsInterface): Promise<{}> {
        return this._showDialog( Object.assign(options, { status: 'warning'}) );
    }



    public error(options: ModalArgumentsInterface): Promise<{}> {
        return this._showDialog( Object.assign(options, { status: 'error'}) );
    }

    public dialog_question(options: ModalArgumentsInterface): Promise<{}> {
        return this._showDialog( Object.assign(options, { status: 'dialog_question'}) );
    }

    public filter_modal(options: ModalArgumentsInterface): Promise<{}> {
        return this._showFilter( Object.assign(options, { status: 'filter_modal'}) );
    }



    private _showDialog(context: ModalArgumentsInterface): Promise<{}> {
        if (context.message === 'Check the Internet connection') {
            return new Promise(resolve => {});
        }
        this.showModal$.next({
            type:       'dialog',
            context,
            buttonCount: 0
        });

        return new Promise(resolve => {
            context.handler = (arg: any) => resolve(arg);
        });
    }

    private _showFilter(context: ModalArgumentsInterface): Promise<{}> {
        this.showModal$.next({
            type:       'filter',
            context,
            buttonCount: 0
        });

        return new Promise(resolve => {
            context.handler = (arg: any) => resolve(arg);
        });
    }



    public showImage(url: string): void {
        this.showModal$.next({
            type:       'image',
            context:    {
                message:    url
            },
            buttonCount: 0
        });
    }




    public showSpinner(): void {
        this.showModal$.next({
            type:       'spinner',
            context:    {},
            buttonCount: 0
        });
    }




    public showUpload(): void {
        this.showModal$.next({
            type:       'upload',
            context:    {},
            buttonCount: 0
        });
    }



    getCommand() {
        return this.showModal$.asObservable();
    }

}
