import {
    Component,
    ElementRef,
    EventEmitter,
    Input,
    Output,
    ViewChild
} from '@angular/core';
import {Observable, OperatorFunction} from 'rxjs';
import {debounceTime, distinctUntilChanged, switchMap} from 'rxjs/operators';
import {DatasetIDsInterface, TypeNames} from '../../interface/search.interface';
import {SearchApi} from '../../api/search.api';
import {UserInterface} from '../../interface/auth.interface';
import AppValues from "../../common/app.values";

@Component({
  selector: 'app-header',
  templateUrl: './app-header.component.html'
})
export class AppHeaderComponent {
    @Input() public searchValue: DatasetIDsInterface = {id: '', __typename: TypeNames.allDataType};
    @Input() public appLogo: string;
    @Input() public isMobileView: boolean;
    @Input() public isVisible: boolean;
    @Input() public userInfo: UserInterface;

    @Output() public selectDatasetEmitter: EventEmitter<DatasetIDsInterface> = new EventEmitter();
    @Output() public addNewEmitter: EventEmitter<null> = new EventEmitter();
    @Output() public loginEmitter: EventEmitter<null> = new EventEmitter();
    @Output() public logOutEmitter: EventEmitter<null> = new EventEmitter();
    @Output() public userProfileEmitter: EventEmitter<null> = new EventEmitter();
    @Output() public onClickAppLogoEmitter: EventEmitter<null> = new EventEmitter();

    @ViewChild('appHeaderMenuButton') appHeaderMenuButton: ElementRef<HTMLElement>;

    public defaultUsername: string = AppValues.defaultUsername;
    public isSearchActive = false;
    private _window: Window;

    constructor(private appSearchAPI: SearchApi) {
        this._window = window;
    }

    public isDatasetType(type: string): boolean {
        return type === TypeNames.datasetType;
    }
    public search: OperatorFunction<string, readonly DatasetIDsInterface[]> = (text$: Observable<string>) => {
        return text$.pipe(
            debounceTime(300),
            distinctUntilChanged(),
            switchMap(term => this.appSearchAPI.autocompleteDatasetSearch(term)));
    }

    public formatter(x: DatasetIDsInterface | string): string {
        return typeof x !== 'string' ? x.id : x;
    }

    public onClickInput(): void {
        const typeaheadInput: HTMLElement | null = document.getElementById('typeahead-http');
        if (typeaheadInput) {
            typeaheadInput.focus();
        }
    }
    public onSelectItem(event: any): void {
        this.isSearchActive = false;

        if (event.item) {
            this.selectDatasetEmitter.emit(event.item);

            setTimeout(() => {
                const typeaheadInput: HTMLElement | null = document.getElementById('typeahead-http');
                if (typeaheadInput) {
                    typeaheadInput.blur();
                }
            });
        }
    }

    public onSearch(event: any, searchValue: DatasetIDsInterface | string): void {
        this.isSearchActive = false;

        setTimeout(() => {
            if (this.isMobileView) {
                this.triggerMenuClick();
            }

            (event.target as HTMLElement).blur();
            const typeaheadInput: Element | null = document.querySelector('ngb-typeahead-window');
            if (typeaheadInput) {
                // eslint-disable-next-line @typescript-eslint/ban-ts-comment
                // @ts-ignore
                document.querySelector('ngb-typeahead-window').classList.remove('show');
            }
        }, 200);
    }

    public onLogin(): void {
        this.loginEmitter.emit();
    }

    public onLogOut(): void {
        this.logOutEmitter.emit();
    }

    public onAddNew(): void {
        this.addNewEmitter.emit();
    }
    public onOpenUserInfo(): void {
        this.userProfileEmitter.emit();
    }

    public triggerMenuClick(): void {
        const el: HTMLElement = this.appHeaderMenuButton.nativeElement;
        el.focus();
        el.click();
        el.blur();
    }
    public onClickAppLogo(): void {
        this.onClickAppLogoEmitter.emit();
    }
}
