import {Component, HostListener, OnInit} from '@angular/core';
import AppValues from "./common/app.values";
import {AppSearchService} from "./search/search.service";
import { filter } from 'rxjs/operators';
import {Router, NavigationEnd} from '@angular/router';
import {DatasetIDsInterface} from "./interface/search.interface";

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.sass']
})
export class AppComponent implements OnInit {
  public appLogo: string = `/${AppValues.appLogo}`;
  public isMobileView: boolean = false;
  public searchValue: string = '';
  public isVisible: boolean = true;
  public ngTypeaheadList: DatasetIDsInterface[] = [];
  private appHeaderNotVisiblePages: string[] = [AppValues.urlDatasetCreate, AppValues.urlLogin];
  private _window: Window;

  @HostListener('window:resize', ['$event'])
  private checkWindowSize(): void {
    this.checkView();
  }

  constructor(
      private router: Router,
      private appSearchService: AppSearchService,
  ) {
    this._window = window;
  }

  public ngOnInit(): void {
    this.checkView();
    this.appHeaderInit();
  }
  private appHeaderInit(): void {
    this.router.events
        .pipe(filter(event => event instanceof NavigationEnd))
        .subscribe((event: any) => {
          this.isVisible = this.isAvailableAppHeaderUrl(event.url);
        });
    this.appSearchService.onSearchChanges.subscribe((searchValue: string) => {
      this.searchValue = searchValue;
      this.searchValueAddToAutocomplete();
    });
    this.appSearchService.onAutocompleteDatasetChanges.subscribe((data: DatasetIDsInterface[]) => {
        this.ngTypeaheadList = data;
        this.searchValueAddToAutocomplete();
    });
  }

  private searchValueAddToAutocomplete(): void {
     let newArray: DatasetIDsInterface[] = JSON.parse(JSON.stringify(this.ngTypeaheadList));
      newArray.unshift({__typename: "Dataset", id: this.searchValue});
      this.ngTypeaheadList = newArray;
  }

  private checkView(): void {
    this.isMobileView = AppValues.isMobileView();
  }
  private isAvailableAppHeaderUrl(url: string): boolean {
     return !this.appHeaderNotVisiblePages.some(item => url.toLowerCase().includes(item));
  }
  public onInputSearch(searchValue: string) {
    debugger
    if (!this._window.location.pathname.includes(AppValues.urlSearch)) {
      this.router.navigate(['search']);
    }
    this.appSearchService.search(searchValue);
  }

  public onKeyUpSearch(searchValue: string) {
    this.appSearchService.searchChanges(searchValue);
    this.appSearchService.autocompleteDatasetSearch(searchValue);
  }

  public onOpenUserInfo(): void {
    console.info('click onOpenUserInfo');
  }

  public onAddNew(): void {
    this.router.navigate(['dataset-create'])
  }
}
