import {AfterContentInit, Component, HostListener, OnInit, ViewChild} from '@angular/core';
import {MatTableDataSource} from '@angular/material/table';
import {AppSearchService} from "./search.service";
import {SearchHistoryInterface} from "../interface/search.interface";
import AppValues from "../common/app.values";
import {searchAdditionalButtonsEnum} from "./search.interface";
import {SearchAdditionalButtonInterface} from "../components/search-additional-buttons/search-additional-buttons.interface";
import {MatSidenav} from "@angular/material/sidenav";
import {SideNavService} from "../services/sidenav.service";
import {Router} from "@angular/router";


const ELEMENT_DATA: any[] = [];

@Component({
  selector: 'app-search',
  templateUrl: './search.component.html',
  styleUrls: ['./search.component.sass']
})
export class SearchComponent implements OnInit, AfterContentInit {

  @ViewChild('sidenav', {static: true}) public sidenav?: MatSidenav;
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
  public tableData: {
    isTableHeader: boolean,
    displayedColumns?: any[],
    tableSource: any,
    isResultQuantity: boolean
  };
  public searchData: SearchHistoryInterface[] = [];

  @HostListener('window:resize', ['$event'])
  private checkWindowSize(): void {
    this.isMinimizeSearchAdditionalButtons = AppValues.isMobileView();
    this.isMobileView = AppValues.isMobileView();

    if (AppValues.isMobileView()) {
      this.sidenavService.close();
    } else {
      this.sidenavService.open();
    }
  }
  // public dataSource = new MatTableDataSource<any>(ELEMENT_DATA);

  constructor(
      private router: Router,
      private appSearchService: AppSearchService,
      private sidenavService: SideNavService) {
  }

  public ngAfterContentInit(): void {
    this.tableData.tableSource = this.searchData
  }


  public ngOnInit(): void {
    if(this.sidenav) {
      this.sidenavService.setSidenav(this.sidenav);
      this.checkWindowSize();
    }

    this.initTableData();
    this.onSearch("");
    this.appSearchService.onSearchChanges.subscribe((value: string) => {
      this.searchValue = value;
    })

    this.appSearchService.onSearchDataChanges.subscribe((data: any[]) => {
      this.tableData.tableSource = data;
    });
  }

  private initTableData(): void {
    this.tableData = {
      isTableHeader: false,
      tableSource: this.searchData,
      isResultQuantity: true
    };
  }

  public onSelectDataset(dataset: any): void {
    this.router.navigate([`dataset-view/${dataset.id}`]);
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

  // private renderTable(data: SearchHistoryInterface[]): void {
  //   const elementsData: SearchHistoryInterface[] = [];
  //   if (!data.length) {
  //     this.dataSource.data = [];
  //     return;
  //   }
  //   this.dataSource.data = [];
  //   const keys_data: string[] = Object.keys(data[0]);
  //
  //   this.displayedColumns = keys_data;
  //
  //   const dataSource = this.dataSource.data;
  //   data.forEach((field: SearchHistoryInterface) => {
  //     dataSource.push(field);
  //   })
  //   this.dataSource.data = dataSource;
  // }

  public showHistory(): void {
    this.appSearchService.searchHistory();
  }

  public onSearch(searchValue: string): void {
    this.appSearchService.search(searchValue);
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
    if (this.sidenavService.isSidenavOpened() && AppValues.isMobileView()) {
      this.onToggleSidenav();
    }
  }

  public onInputSearch(value: string): void {
    this.appSearchService.searchChanges(value);
    this.onSearch(value);
  }
  public onOpenUserInfo(): void {
    console.info('click onOpenUserInfo');
  }
  public onAddNew(): void {
    console.info('click onAddNew');
  }

}