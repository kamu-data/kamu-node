import {Component, HostListener, OnInit, ViewChild} from '@angular/core';
import {MatTableDataSource} from '@angular/material/table';
import {AppSearchService} from "./search.service";
import {SearchHistoryInterface} from "../interface/search.interface";
import AppValues from "../common/app.values";
import {searchAdditionalButtonsEnum} from "./search.interface";
import {SearchAdditionalButtonInterface} from "../components/search-additional-buttons/search-additional-buttons.interface";
import {MatSidenav} from "@angular/material/sidenav";
import {SideNavService} from "../services/sidenav.service";

const ELEMENT_DATA: SearchHistoryInterface[] = [];

@Component({
  selector: 'app-search',
  templateUrl: './search.component.html',
  styleUrls: ['./search.component.sass']
})
export class SearchComponent implements OnInit {

  @ViewChild('sidenav', {static: true}) public sidenav?: MatSidenav;
  public appLogo: string = `/${AppValues.appLogo}`;
  public isMobileView: boolean = false;
  public searchValue: string = '';
  public isMinimizeSearchAdditionalButtons: boolean = false;
  public searchAdditionalButtonsData: SearchAdditionalButtonInterface[] = [{
    textButton: searchAdditionalButtonsEnum.Descission
  }, {
    textButton: searchAdditionalButtonsEnum.Reputation
  }, {
    textButton: searchAdditionalButtonsEnum.Explore,
    styleClassContainer: 'app-active-button__container',
    styleClassButton: 'app-active-button'
  }, {
    textButton: searchAdditionalButtonsEnum.DeriveForm,
    styleClassContainer: 'app-active-button__container',
    styleClassButton: 'app-active-button'
  }];

  public displayedColumns: string[] = [];
  public dataSource = new MatTableDataSource<any>(ELEMENT_DATA);

  @HostListener('window:resize', ['$event'])
  private checkWindowSize(): void {
    this.isMinimizeSearchAdditionalButtons = (window.innerWidth < window.innerHeight);

    debugger
    this.isMobileView = (window.innerWidth < window.innerHeight);

    if (window.innerWidth < window.innerHeight) {
      this.sidenavService.close();
    } else {
      this.sidenavService.open();
    }
  }

  constructor(
      private appSearchService: AppSearchService,
      private sidenavService: SideNavService) {
  }


  public ngOnInit(): void {
    if(this.sidenav) {
      this.sidenavService.setSidenav(this.sidenav);
      this.checkWindowSize();
    }

    this.onSearch();
    this.appSearchService.onSearchChanges.subscribe((value: string) => {
      this.searchValue = value;
    })

    this.appSearchService.onSearchDataChanges.subscribe((data: any[]) => {
      this.renderTable(data);
    });
  }


  public onClickSearchAdditionalButton(method: string) {
    if (method === searchAdditionalButtonsEnum.DeriveForm) {
      this.onClickDeriveForm();
    }
    if (method === searchAdditionalButtonsEnum.Reputation) {
      this.onClickReputation();
    }
    if (method === searchAdditionalButtonsEnum.Explore) {
      this.onClickExplore();
    }
    if (method === searchAdditionalButtonsEnum.Descission) {
      this.onClickDescission();
    }
  }

  private onClickDeriveForm() {
  }
  private onClickExplore() {
  }
  private onClickReputation() {
  }
  private onClickDescission() {
  }

  private renderTable(data: SearchHistoryInterface[]): void {
    const elementsData: SearchHistoryInterface[] = [];
    if (!data.length) {
      this.dataSource.data = [];
    }
    this.dataSource.data = [];
    const keys_data: string[] = Object.keys(data[0]);

    this.displayedColumns = keys_data;

    const dataSource = this.dataSource.data;
    data.forEach((field: SearchHistoryInterface) => {
      dataSource.push(field);
    })
    this.dataSource.data = dataSource;
  }

  public changeColumnName(columnName: string): string {
    columnName = columnName.replace('_', ' ');
    return AppValues.capitalizeFirstLetter(columnName);
  }

  public showHistory(): void {
    this.appSearchService.searchHistory();
  }

  public onSearch(): void {
    this.appSearchService.search();
  }
  public onSearchProjections(): void {
    this.appSearchService.searchLastTenFields();
  }
  public onSearchMetadata(): void {
    this.appSearchService.onSearchMetadata();
  }

  public onToggleSidenav(): void {
    this.sidenavService.toggle();
  }
  public onCloseSidenav(): void {
    if (this.sidenavService.isSidenavOpened()) {
      this.onToggleSidenav();
    }
  }

  public onInputSearch(value: string): void {
    this.appSearchService.searchChanges(value);
  }
  public onOpenUserInfo(): void {
    console.info('click onOpenUserInfo');
  }
  public onAddNew(): void {
    console.info('click onAddNew');
  }

}